const UPLOAD_CONCURRENCY: usize = 20;

pub enum PushOutcome {
    Pushed { objects_uploaded: usize },
    AlreadyUpToDate,
}

pub async fn push_to_repo(
    path: &str,
    repo_name: &str,
    contract: &ContractAbiClient,
    progress: Option<&ProgressBar>,
) -> Result<PushOutcome> {
    let repo = gix::open(path)?;
    let local_head = repo.head_id()?.detach();

    let Some(repo_info) = contract.state().await.repo_store.get(&repo_name.to_string()).await
    else {
        return Err(VastrumGitError::RepoNotFound(repo_name.to_string()));
    };
    let no_commit_yet = repo_info.head_commit_hash.is_none();

    if no_commit_yet {
        let objects = collect_all_objects(&repo, local_head, None)?;
        let uploaded = upload_objects_concurrent(&objects, contract, progress).await?;
        update_vastrum_head(local_head, repo_name, contract).await;
        return Ok(PushOutcome::Pushed { objects_uploaded: uploaded });
    } else {
        let vastrum_head = sha1_to_oid(&repo_info.head_commit_hash.unwrap());

        if local_head == vastrum_head {
            return Ok(PushOutcome::AlreadyUpToDate);
        } else if local_is_ancestor(vastrum_head, local_head, &repo) {
            let objects = collect_all_objects(&repo, local_head, Some(vastrum_head))?;
            let uploaded = upload_objects_concurrent(&objects, contract, progress).await?;
            update_vastrum_head(local_head, repo_name, contract).await;
            return Ok(PushOutcome::Pushed { objects_uploaded: uploaded });
        } else {
            return Err(VastrumGitError::Diverged);
        }
    }
}

/// Check if potential_ancestor is an ancestor of descendant using local repo
fn local_is_ancestor(
    potential_ancestor: ObjectId,
    descendant: ObjectId,
    repo: &Repository,
) -> bool {
    if potential_ancestor == descendant {
        return true;
    }

    let mut to_visit = vec![descendant];
    let mut visited = HashSet::new();

    while let Some(current) = to_visit.pop() {
        if current == potential_ancestor {
            return true;
        }

        if visited.contains(&current) {
            continue;
        }
        visited.insert(current);

        if let Ok(commit) = repo.find_commit(current) {
            for parent_id in commit.parent_ids() {
                to_visit.push(parent_id.detach());
            }
        }
    }

    return false;
}

pub async fn update_vastrum_head(head: ObjectId, repo_name: &str, contract: &ContractAbiClient) {
    let hash = oid_to_sha1(head);
    contract.set_head_commit(repo_name, hash).await.await_confirmation().await;
}

fn collect_all_objects(
    repo: &Repository,
    head_oid: ObjectId,
    stop_at: Option<ObjectId>,
) -> Result<Vec<Object>> {
    let mut objects = Vec::new();
    let mut visited = HashSet::new();
    collect_commit_graph(repo, head_oid, &mut objects, &mut visited, stop_at)?;
    return Ok(objects);
}

fn collect_commit_graph(
    repo: &Repository,
    oid: ObjectId,
    objects: &mut Vec<Object>,
    visited: &mut HashSet<ObjectId>,
    stop_at: Option<ObjectId>,
) -> Result<()> {
    if !visited.insert(oid) {
        return Ok(());
    }
    //if is att latest head_commit then stop collecting as can assume
    //all head commit data is already uploaded
    if stop_at == Some(oid) {
        return Ok(());
    }

    let commit = repo.find_object(oid)?.to_commit_ref().to_owned()?;

    collect_tree_objects(repo, commit.tree, objects, visited)?;

    objects.push(Object::Commit(commit.clone()));

    for parent in &commit.parents {
        collect_commit_graph(repo, *parent, objects, visited, stop_at)?;
    }
    return Ok(());
}

fn collect_tree_objects(
    repo: &Repository,
    tree_oid: ObjectId,
    objects: &mut Vec<Object>,
    visited: &mut HashSet<ObjectId>,
) -> Result<()> {
    if !visited.insert(tree_oid) {
        return Ok(());
    }

    let tree = repo.find_tree(tree_oid)?.to_owned().decode()?.into_owned();

    objects.push(Object::Tree(tree.clone()));

    for entry in &tree.entries {
        let kind = entry.mode.kind();
        if kind == EntryKind::Tree {
            collect_tree_objects(repo, entry.oid, objects, visited)?;
        } else if kind == EntryKind::Commit {
            return Err(VastrumGitError::SubmodulesNotSupported);
        } else {
            if !visited.insert(entry.oid) {
                continue;
            }
            let obj = repo.find_object(entry.oid)?;
            let data = obj.into_blob().take_data();
            objects.push(Object::Blob(Blob { data }));
        }
    }
    return Ok(());
}

async fn upload_objects_concurrent(
    objects: &[Object],
    contract: &ContractAbiClient,
    progress: Option<&ProgressBar>,
) -> Result<usize> {
    // Very complex in order to parallelize uploads and do error reporting + progress reporting

    //Basically only this though
    //let loose = serialize_git_object(obj);
    //contract.upload_git_object(loose).await;

    // Filter to only upload objects not yet uploaded onchain
    if let Some(progress) = progress {
        progress.set_message("Checking".to_string());
        progress.set_length(objects.len() as u64);
        progress.set_position(0);
    }
    let mut to_upload: Vec<&Object> = Vec::new();
    for obj in objects {
        let oid = calculate_object_hash(obj);
        let key = oid_to_sha1(oid);
        if contract.state().await.git_object_store.get(&key).await.is_none() {
            to_upload.push(obj);
        }
        if let Some(progress) = progress {
            progress.inc(1);
        }
    }

    let upload_count = to_upload.len();
    if upload_count == 0 {
        return Ok(0);
    }
    if let Some(progress) = progress {
        progress.set_message("Uploading".to_string());
        progress.set_length(upload_count as u64);
        progress.set_position(0);
    }

    for chunk in to_upload.chunks(UPLOAD_CONCURRENCY) {
        let mut serialized = Vec::with_capacity(chunk.len());
        for obj in chunk {
            let loose = serialize_git_object(obj);
            if loose.len() > MAX_TRANSACTION_SIZE {
                return Err(VastrumGitError::ObjectTooLarge {
                    oid: calculate_object_hash(obj).to_string(),
                    size: loose.len(),
                    max: MAX_TRANSACTION_SIZE,
                });
            }
            serialized.push(loose);
        }
        let sent_txs = futures::future::join_all(
            serialized.into_iter().map(|loose| contract.upload_git_object(loose)),
        )
        .await;
        let mut set: tokio::task::JoinSet<()> = tokio::task::JoinSet::new();
        for tx in sent_txs {
            set.spawn(async move { tx.await_confirmation().await });
        }
        while let Some(res) = set.join_next().await {
            res.expect("confirmation task panicked");
            if let Some(progress) = progress {
                progress.inc(1);
            }
        }
    }

    return Ok(upload_count);
}
use crate::{
    ContractAbiClient,
    error::{Result, VastrumGitError},
    universal::utils::{calculate_object_hash, oid_to_sha1, serialize_git_object, sha1_to_oid},
};
use gix::{
    ObjectId, Repository,
    object::tree::EntryKind,
    objs::{Blob, Object},
};
use indicatif::ProgressBar;
use std::collections::HashSet;
use vastrum_rpc_client::SentTxBehavior;
use vastrum_shared_types::limits::MAX_TRANSACTION_SIZE;

#[cfg(test)]
mod tests {
    use gix::{ObjectId, Repository};
    use serial_test::serial;
    use vastrum_rpc_client::SentTxBehavior;

    #[tokio::test]
    #[serial]
    async fn test_repo_creation() {
        let ctx = TestContext::new().await;

        let repo_name = "repo1";

        ctx.contract.create_repository(repo_name, "testdescr").await.await_confirmation().await;

        let value =
            ctx.contract.state().await.repo_store.get(&repo_name.to_string()).await.unwrap();
        assert_eq!(value.name, repo_name);
        assert!(value.head_commit_hash.is_none());
        assert_eq!(value.owner, ctx.account_key.public_key());

        ctx.contract.create_repository(repo_name, "").await.await_confirmation().await;

        let value =
            ctx.contract.state().await.repo_store.get(&repo_name.to_string()).await.unwrap();
        assert_eq!(value.name, repo_name);
        assert!(value.head_commit_hash.is_none());
        assert_eq!(value.owner, ctx.account_key.public_key());

        ctx.contract.create_repository("reponame2", "").await.await_confirmation().await;

        let zero_repo = ctx.contract.state().await.all_repos.get(0).await.unwrap();
        let one_repo = ctx.contract.state().await.all_repos.get(1).await.unwrap();
        assert_eq!(zero_repo.name, repo_name);
        assert_eq!(one_repo.name, "reponame2");
    }

    #[tokio::test]
    #[serial]
    async fn test_repo_roundtrip_push_clone() {
        let ctx = TestContext::new().await;
        let repo_name = "repo1";
        let source_path = "test_repos/test_repo";
        let target_path = "test_repos/__out_repo";

        if std::fs::exists(&target_path).unwrap() {
            std::fs::remove_dir_all(&target_path).unwrap();
        }
        ctx.contract.create_repository(repo_name, "description").await.await_confirmation().await;

        push_to_repo(source_path, repo_name, &ctx.contract, None).await.unwrap();
        clone_repo(repo_name, target_path, &ctx.contract, None).await.unwrap();

        assert_repos_equal(source_path, target_path);
    }

    fn assert_repos_equal(source_path: &str, target_path: &str) {
        let source = gix::open(source_path).unwrap();
        let target = gix::open(target_path).unwrap();

        let source_head = source.head().unwrap().id().unwrap().detach();
        let target_head = target.head().unwrap().id().unwrap().detach();
        assert_eq!(source_head, target_head, "HEAD commits differ");

        assert_commits_equal(&source, &target, source_head);
    }

    fn assert_commits_equal(source: &Repository, target: &Repository, oid: ObjectId) {
        let src_commit = source.find_commit(oid).unwrap();
        target.find_commit(oid).unwrap();

        assert_trees_equal(source, target, src_commit.tree_id().unwrap().detach());

        for parent_id in src_commit.parent_ids() {
            assert_commits_equal(source, target, parent_id.detach());
        }
    }

    fn assert_trees_equal(source: &Repository, target: &Repository, tree_id: ObjectId) {
        let src_tree = source.find_tree(tree_id).unwrap();
        let tgt_tree = target.find_tree(tree_id).unwrap();

        let src_entries: Vec<_> = src_tree.iter().collect();
        let tgt_entries: Vec<_> = tgt_tree.iter().collect();
        assert_eq!(
            src_entries.len(),
            tgt_entries.len(),
            "Tree entry count differs for {}",
            tree_id
        );

        for (src_entry, tgt_entry) in src_entries.iter().zip(tgt_entries.iter()) {
            let src_entry = src_entry.as_ref().unwrap();
            let tgt_entry = tgt_entry.as_ref().unwrap();

            assert_eq!(src_entry.id(), tgt_entry.id(), "Entry ID differs");
            assert_eq!(src_entry.mode(), tgt_entry.mode(), "Entry mode differs");

            if src_entry.mode().is_tree() {
                assert_trees_equal(source, target, src_entry.id().detach());
            } else if src_entry.mode().is_blob() {
                let src_blob = source.find_blob(src_entry.id()).unwrap();
                let tgt_blob = target.find_blob(tgt_entry.id()).unwrap();
                assert_eq!(src_blob.data, tgt_blob.data, "Blob content differs");
            }
        }
    }
    use crate::{
        native::{clone::clone_repo, upload::push_to_repo},
        testing::test_helpers::TestContext,
    };
}
