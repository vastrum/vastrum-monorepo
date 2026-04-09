pub fn oid_to_sha1(oid: ObjectId) -> Sha1Hash {
    let hash = Sha1Hash(oid.as_slice().try_into().unwrap());
    return hash;
}

pub fn sha1_to_oid(hash: &Sha1Hash) -> ObjectId {
    let oid = ObjectId::from(hash.0);
    return oid;
}

pub async fn vastrum_get_head_commit(
    repo_name: impl Into<String>,
    contract: &ContractAbiClient,
) -> Result<ObjectId> {
    let repo_name = repo_name.into();

    let state = contract.state().await;
    let Some(repo) = state.repo_store.get(&repo_name).await else {
        return Err(VastrumGitError::RepoNotFound(repo_name.clone()));
    };

    let Some(hash) = repo.head_commit_hash else {
        return Err(VastrumGitError::RepoDoesNotHaveHeadCommitYet);
    };

    return Ok(sha1_to_oid(&hash));
}

pub async fn publickey_is_owner_of_repo(
    repo_name: &str,
    public_key: PublicKey,
    contract: &ContractAbiClient,
) -> Result<bool> {
    let Some(repo) = contract.state().await.repo_store.get(&repo_name.to_string()).await else {
        return Err(VastrumGitError::RepoNotFound(repo_name.to_string()));
    };
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

use crate::ContractAbiClient;
use crate::Sha1Hash;
use crate::error::{Result, VastrumGitError};
use gix_hash::{Kind, ObjectId};
use gix_object::{
    Blob, Commit, Object, ObjectRef, Tree, WriteTo, compute_hash, encode, tree::EntryMode,
};
use std::collections::{HashMap, HashSet};
use vastrum_rpc_client::SentTxBehavior;
use vastrum_shared_types::crypto::ed25519::PublicKey;

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
        // Build ancestor set for ours
        let mut ours_ancestors = HashSet::new();
        let mut to_visit = vec![ours];

        while let Some(current) = to_visit.pop() {
            if ours_ancestors.contains(&current) {
                continue;
            }
            ours_ancestors.insert(current);

            let commit = self.read_commit(current).await?;
            for parent in commit.parents.iter() {
                if !ours_ancestors.contains(parent) {
                    to_visit.push(*parent);
                }
            }
        }

        // Walk theirs ancestors until finding common commit
        let mut to_visit = vec![theirs];
        let mut visited = HashSet::new();

        while let Some(current) = to_visit.pop() {
            if ours_ancestors.contains(&current) {
                return Ok(Some(current));
            }

            if visited.contains(&current) {
                continue;
            }
            visited.insert(current);

            let commit = self.read_commit(current).await?;
            for parent in commit.parents.iter() {
                if !visited.contains(parent) {
                    to_visit.push(*parent);
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
