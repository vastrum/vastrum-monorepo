pub fn oid_to_sha1(oid: ObjectId) -> Sha1Hash {
    let hash = Sha1Hash(oid.as_slice().try_into().unwrap());
    return hash;
}

pub fn sha1_to_oid(hash: &Sha1Hash) -> ObjectId {
    let oid = ObjectId::from(hash.0);
    return oid;
}

pub async fn get_repo(
    repo_store: &vastrum_abi::__private::vastrum_native_types::KvMap<String, crate::GitRepository>,
    repo_name: &str,
) -> Result<crate::GitRepository> {
    let Some(repo) = repo_store.get(&repo_name.to_string()).await else {
        return Err(VastrumGitError::RepoNotFound(repo_name.to_string()));
    };
    return Ok(repo);
}

pub async fn get_head_commit(
    repo_store: &vastrum_abi::__private::vastrum_native_types::KvMap<String, crate::GitRepository>,
    repo_name: &str,
    branch: &str,
) -> Result<ObjectId> {
    let repo = get_repo(repo_store, repo_name).await?;
    let Some(hash) = repo.branches.get(branch) else {
        return Err(VastrumGitError::RepoDoesNotHaveHeadCommitYet);
    };
    return Ok(sha1_to_oid(hash));
}

pub async fn get_default_branch_head_commit(
    repo_store: &vastrum_abi::__private::vastrum_native_types::KvMap<String, crate::GitRepository>,
    repo_name: &str,
) -> Result<ObjectId> {
    let repo = get_repo(repo_store, repo_name).await?;
    let Some(hash) = repo.branches.get(&repo.default_branch) else {
        return Err(VastrumGitError::RepoDoesNotHaveHeadCommitYet);
    };
    return Ok(sha1_to_oid(hash));
}

pub async fn publickey_is_owner_of_repo(
    repo_name: &str,
    public_key: PublicKey,
    contract: &ContractAbiClient,
) -> Result<bool> {
    let repo = get_repo(&contract.state().await.repo_store, repo_name).await?;
    return Ok(repo.owner == public_key);
}

pub fn serialize_git_object(object: &Object) -> Vec<u8> {
    let mut content = Vec::new();
    object.write_to(&mut content).unwrap();
    let header = encode::loose_header(object.kind(), content.len() as u64);
    let mut encoded_bytes = Vec::with_capacity(header.len() + content.len());
    encoded_bytes.extend_from_slice(&header);
    encoded_bytes.extend_from_slice(&content);
    return encoded_bytes;
}

pub async fn upload_git_object(object: Object, contract: &ContractAbiClient) {
    let loose = serialize_git_object(&object);
    contract.upload_git_object(loose).await.await_confirmation().await;
}

pub fn calculate_object_hash(object: &Object) -> ObjectId {
    let mut buf = Vec::new();
    object.write_to(&mut buf).unwrap();
    let oid = compute_hash(Kind::Sha1, object.kind(), &buf).unwrap();
    return oid;
}

pub struct GitContext {
    store: vastrum_abi::__private::vastrum_native_types::KvMap<Sha1Hash, Vec<u8>>,
}

impl GitContext {
    pub fn new(
        store: vastrum_abi::__private::vastrum_native_types::KvMap<Sha1Hash, Vec<u8>>,
    ) -> Self {
        Self { store }
    }

    pub async fn from_contract(contract: &ContractAbiClient) -> Self {
        Self { store: contract.state().await.git_object_store }
    }

    pub async fn object_read(&self, oid: ObjectId) -> Result<Object> {
        let key = oid_to_sha1(oid);
        let Some(bytes) = self.store.get(&key).await else {
            return Err(VastrumGitError::ObjectNotFound(oid.to_string()));
        };
        let Ok(obj_ref) = ObjectRef::from_loose(&bytes) else {
            return Err(VastrumGitError::LooseDecode(oid.to_string()));
        };
        return Ok(obj_ref.into_owned()?);
    }

    pub async fn read_commit(&self, commit_id: ObjectId) -> Result<Commit> {
        Ok(self.object_read(commit_id).await?.into_commit())
    }

    pub async fn read_tree(&self, tree_id: ObjectId) -> Result<Tree> {
        Ok(self.object_read(tree_id).await?.into_tree())
    }

    pub async fn blob_read(&self, blob_id: ObjectId) -> Result<Blob> {
        Ok(self.object_read(blob_id).await?.into_blob())
    }

    pub async fn read_tree_entries(
        &self,
        tree_id: Option<ObjectId>,
    ) -> Result<HashMap<String, (ObjectId, EntryMode)>> {
        match tree_id {
            None => Ok(HashMap::new()),
            Some(id) => {
                let tree = self.read_tree(id).await?;
                let mut entries = HashMap::new();
                for e in tree.entries {
                    entries.insert(e.filename.to_string(), (e.oid, e.mode));
                }
                Ok(entries)
            }
        }
    }
    /// Find the merge base (common ancestor) of two commits
    pub async fn find_merge_base(
        &self,
        ours: ObjectId,
        theirs: ObjectId,
    ) -> Result<Option<ObjectId>> {
        if ours == theirs {
            return Ok(Some(ours));
        }

        let mut visited_ours: HashSet<ObjectId> = HashSet::from([ours]);
        let mut visited_theirs: HashSet<ObjectId> = HashSet::from([theirs]);
        let mut frontier_ours: VecDeque<ObjectId> = VecDeque::from([ours]);
        let mut frontier_theirs: VecDeque<ObjectId> = VecDeque::from([theirs]);

        while !frontier_ours.is_empty() || !frontier_theirs.is_empty() {
            if let Some(current) = frontier_ours.pop_front() {
                let commit = self.read_commit(current).await?;
                for parent in commit.parents.iter() {
                    if visited_theirs.contains(parent) {
                        return Ok(Some(*parent));
                    }
                    if visited_ours.insert(*parent) {
                        frontier_ours.push_back(*parent);
                    }
                }
            }
            if let Some(current) = frontier_theirs.pop_front() {
                let commit = self.read_commit(current).await?;
                for parent in commit.parents.iter() {
                    if visited_ours.contains(parent) {
                        return Ok(Some(*parent));
                    }
                    if visited_theirs.insert(*parent) {
                        frontier_theirs.push_back(*parent);
                    }
                }
            }
        }

        return Ok(None);
    }

    /// Get all commit OIDs from base (exclusive) to head (inclusive)
    pub async fn get_commits_since(&self, base: ObjectId, head: ObjectId) -> Result<Vec<ObjectId>> {
        let mut commits = Vec::new();
        let mut to_visit = vec![head];
        let mut visited = HashSet::new();

        while let Some(current) = to_visit.pop() {
            // Stop at the base commit (don't include it)
            if current == base {
                continue;
            }
            if visited.contains(&current) {
                continue;
            }
            visited.insert(current);
            commits.push(current);

            let commit = self.read_commit(current).await?;
            for parent in commit.parents.iter() {
                if !visited.contains(parent) {
                    to_visit.push(*parent);
                }
            }
        }

        return Ok(commits);
    }

    /// Get all commits on a feature branch since it diverged from the base branch
    pub async fn get_feature_commits(
        &self,
        base_head: ObjectId,
        feature_head: ObjectId,
    ) -> Result<Vec<Commit>> {
        let Some(base) = self.find_merge_base(base_head, feature_head).await? else {
            return Ok(vec![]);
        };
        let oids = self.get_commits_since(base, feature_head).await?;
        let mut commits = Vec::with_capacity(oids.len());
        for oid in &oids {
            commits.push(self.read_commit(*oid).await?);
        }
        return Ok(commits);
    }
}

use crate::ContractAbiClient;
use crate::Sha1Hash;
use crate::error::{Result, VastrumGitError};
use gix_hash::{Kind, ObjectId};
use gix_object::{
    Blob, Commit, Object, ObjectRef, Tree, WriteTo, compute_hash, encode, tree::EntryMode,
};
use std::collections::{HashMap, HashSet, VecDeque};
use vastrum_rpc_client::SentTxBehavior;
use vastrum_shared_types::crypto::ed25519::PublicKey;
