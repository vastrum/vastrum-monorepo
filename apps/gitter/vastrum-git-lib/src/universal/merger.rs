pub async fn merge_repos(
    repo_name: impl Into<String>,
    fork_repo: impl Into<String>,
    contract: &ContractAbiClient,
    mode: MergeMode,
) -> Result<MergeResult> {
    let repo_name: String = repo_name.into();
    let fork_repo: String = fork_repo.into();
    let ctx = GitContext::from_contract(contract).await;

    let ours_commit_id = vastrum_get_head_commit(&repo_name, contract).await?;
    let theirs_commit_id = vastrum_get_head_commit(&fork_repo, contract).await?;
    let res = merge_branches(ours_commit_id, theirs_commit_id, &ctx, contract, mode).await?;

    //only update headcommit if mergemode is live
    if mode == MergeMode::Live {
        if let MergeResult::Merged(id) | MergeResult::FastForward(id) = res {
            let hash = oid_to_sha1(id);
            contract.set_head_commit(repo_name, hash).await.await_confirmation().await;
        }
    }
    return Ok(res);
}

pub async fn merge_branches(
    ours_commit_id: ObjectId,
    theirs_commit_id: ObjectId,
    ctx: &GitContext,
    contract: &ContractAbiClient,
    mode: MergeMode,
) -> Result<MergeResult> {
    // Check if theirs is ancestor of ours = already up to date
    let already_up_to_date = is_ancestor(theirs_commit_id, ours_commit_id, ctx).await?;
    if already_up_to_date {
        return Ok(MergeResult::AlreadyUpToDate);
    }

    // Check if ours is ancestor of theirs = fastforward
    let fast_forward = is_ancestor(ours_commit_id, theirs_commit_id, ctx).await?;
    if fast_forward {
        return Ok(MergeResult::FastForward(theirs_commit_id));
    }

    // Perform 3 way merge
    let result = merge_trees(ours_commit_id, theirs_commit_id, ctx, contract, mode).await;
    return result;
}

async fn is_ancestor(
    potential_ancestor: ObjectId,
    descendant: ObjectId,
    ctx: &GitContext,
) -> Result<bool> {
    if potential_ancestor == descendant {
        return Ok(true);
    }

    let mut to_visit = vec![descendant];
    let mut visited = HashSet::new();

    while let Some(current) = to_visit.pop() {
        if current == potential_ancestor {
            return Ok(true);
        }

        if visited.contains(&current) {
            continue;
        }
        visited.insert(current);

        let commit = ctx.read_commit(current).await?;
        for parent in commit.parents.iter() {
            if !visited.contains(parent) {
                to_visit.push(*parent);
            }
        }
    }

    return Ok(false);
}

async fn create_and_upload_tree(
    mut entries: Vec<(String, ObjectId, EntryMode)>,
    contract: &ContractAbiClient,
) -> ObjectId {
    // Sort with git rules (dirs have trailing /)
    entries.sort_by(|(a_name, _, a_mode), (b_name, _, b_mode)| {
        let a = if a_mode.is_tree() { format!("{}/", a_name) } else { a_name.clone() };
        let b = if b_mode.is_tree() { format!("{}/", b_name) } else { b_name.clone() };
        a.cmp(&b)
    });

    let mut tree_entries = vec![];
    for (name, oid, mode) in entries {
        let entry = Entry { mode, filename: name.into(), oid };
        tree_entries.push(entry);
    }

    let tree = Tree { entries: tree_entries };
    let obj = Object::Tree(tree);
    upload_git_object(obj.clone(), contract).await;
    let tree_hash = calculate_object_hash(&obj);
    return tree_hash;
}

async fn merge_entry(
    base: Option<(ObjectId, EntryMode)>,
    ours: Option<(ObjectId, EntryMode)>,
    theirs: Option<(ObjectId, EntryMode)>,
    path: &str,
    conflicts: &mut Vec<ConflictEntry>,
    ctx: &GitContext,
    contract: &ContractAbiClient,
    mode: MergeMode,
) -> Result<Option<(ObjectId, EntryMode)>> {
    let base_oid = base.map(|(oid, _)| oid);
    let ours_oid = ours.map(|(oid, _)| oid);
    let theirs_oid = theirs.map(|(oid, _)| oid);

    if base_oid == ours_oid && ours_oid == theirs_oid {
        return Ok(base);
    }
    if base_oid == ours_oid {
        return Ok(theirs); // Ours unchanged, take theirs
    }
    if base_oid == theirs_oid {
        return Ok(ours); // Theirs unchanged, take ours
    }
    if ours_oid == theirs_oid {
        return Ok(ours); // Both same change
    }

    // Both changed differently, check if we can recurse into directories
    let ours_is_tree = ours.map(|(_, m)| m.is_tree()).unwrap_or(false);
    let theirs_is_tree = theirs.map(|(_, m)| m.is_tree()).unwrap_or(false);

    if ours_is_tree && theirs_is_tree {
        // Both are directories, recurse into them
        let merged = Box::pin(merge_trees_recursive(
            base_oid,
            ours_oid,
            theirs_oid,
            path.to_string(),
            conflicts,
            ctx,
            contract,
            mode,
        ))
        .await?;
        return Ok(merged.map(|oid| (oid, EntryKind::Tree.into())));
    }

    // Conflict: both changed blob differently, or type mismatch (file vs dir)
    // For conflict reporting, use available OIDs (prefer changed versions over base)
    let conflict_ours = ours_oid.or(base_oid).unwrap();
    let conflict_theirs = theirs_oid.or(base_oid).unwrap();

    conflicts.push(ConflictEntry {
        path: path.to_string(),
        ours_oid: conflict_ours,
        theirs_oid: conflict_theirs,
    });

    // Keep ours for now (conflict will be reported)
    return Ok(ours);
}

async fn merge_trees_recursive(
    base: Option<ObjectId>,
    ours: Option<ObjectId>,
    theirs: Option<ObjectId>,
    prefix: String,
    conflicts: &mut Vec<ConflictEntry>,
    ctx: &GitContext,
    contract: &ContractAbiClient,
    mode: MergeMode,
) -> Result<Option<ObjectId>> {
    if base == ours && ours == theirs {
        return Ok(base);
    }
    if base == ours {
        return Ok(theirs); // Ours unchanged, take theirs
    }
    if base == theirs {
        return Ok(ours); // Theirs unchanged, take ours
    }
    if ours == theirs {
        return Ok(ours); // Both same change
    }

    // Both changed differently, need to read and merge entries
    let base_entries = ctx.read_tree_entries(base).await?;
    let ours_entries = ctx.read_tree_entries(ours).await?;
    let theirs_entries = ctx.read_tree_entries(theirs).await?;

    // Collect all entry names from all three trees
    let mut all_names = HashSet::new();
    for name in base_entries.keys().chain(ours_entries.keys()).chain(theirs_entries.keys()) {
        all_names.insert(name.clone());
    }

    let mut merged_entries = Vec::new();

    for name in all_names {
        let base_entry = base_entries.get(&name).copied();
        let ours_entry = ours_entries.get(&name).copied();
        let theirs_entry = theirs_entries.get(&name).copied();
        let path = if prefix.is_empty() { name.clone() } else { format!("{}/{}", prefix, name) };

        if let Some((oid, mode_entry)) =
            merge_entry(base_entry, ours_entry, theirs_entry, &path, conflicts, ctx, contract, mode)
                .await?
        {
            merged_entries.push((name, oid, mode_entry));
        }
        // None means entry was deleted in the merge
    }

    if merged_entries.is_empty() {
        return Ok(None); // Empty tree = deleted directory
    }

    let result = match mode {
        MergeMode::Live => Ok(Some(create_and_upload_tree(merged_entries, contract).await)),
        MergeMode::Preview => Ok(None),
    };
    return result;
}

async fn merge_trees(
    ours_commit_id: ObjectId,
    theirs_commit_id: ObjectId,
    ctx: &GitContext,
    contract: &ContractAbiClient,
    mode: MergeMode,
) -> Result<MergeResult> {
    // Find common ancestor for 3-way merge
    let base_commit_id = match ctx.find_merge_base(ours_commit_id, theirs_commit_id).await? {
        Some(id) => id,
        None => return Ok(MergeResult::NoCommonAncestor),
    };

    // Get tree IDs
    let ours_tree = ctx.read_commit(ours_commit_id).await?.tree;
    let theirs_tree = ctx.read_commit(theirs_commit_id).await?.tree;
    let base_tree = ctx.read_commit(base_commit_id).await?.tree;

    let mut conflicts = Vec::new();

    let result = merge_trees_recursive(
        Some(base_tree),
        Some(ours_tree),
        Some(theirs_tree),
        String::new(),
        &mut conflicts,
        ctx,
        contract,
        mode,
    )
    .await?;

    if !conflicts.is_empty() {
        return Ok(MergeResult::Conflict(conflicts));
    }

    let result = match mode {
        MergeMode::Live => {
            let merged_tree_id = result.expect("merge should produce a tree");
            let merge_commit_id = create_merge_commit(
                merged_tree_id,
                ours_commit_id,
                theirs_commit_id,
                "Vastrum merge",
                contract,
            )
            .await;
            Ok(MergeResult::Merged(merge_commit_id))
        }
        MergeMode::Preview => Ok(MergeResult::Merged(ObjectId::empty_blob(Kind::Sha1))),
    };
    return result;
}

async fn create_merge_commit(
    merged_tree: ObjectId,
    ours_commit: ObjectId,
    theirs_commit: ObjectId,
    message: &str,
    contract: &ContractAbiClient,
) -> ObjectId {
    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs()
        as i64;
    let time = Time::new(now, 0);

    let commit = Commit {
        tree: merged_tree,
        parents: vec![ours_commit, theirs_commit].into(),
        author: Signature {
            name: BString::from("vastrum"),
            email: BString::from("vastrum@vastrum.io"),
            time,
        },
        committer: Signature {
            name: BString::from("vastrum"),
            email: BString::from("vastrum@vastrum.io"),
            time,
        },
        encoding: None,
        message: BString::from(message),
        extra_headers: vec![],
    };

    let commit_object = Object::Commit(commit);
    upload_git_object(commit_object.clone(), contract).await;
    return calculate_object_hash(&commit_object);
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum MergeMode {
    Live,
    Preview,
}

#[derive(PartialEq, Eq, Debug)]
pub enum MergeResult {
    FastForward(ObjectId),
    Merged(ObjectId),
    Conflict(Vec<ConflictEntry>),
    AlreadyUpToDate,
    NoCommonAncestor,
}

/// Represents a merge conflict at a specific path
#[derive(PartialEq, Eq, Debug)]
pub struct ConflictEntry {
    pub path: String,
    pub ours_oid: ObjectId,
    pub theirs_oid: ObjectId,
}

use crate::{
    ContractAbiClient,
    error::Result,
    universal::utils::{
        GitContext, calculate_object_hash, oid_to_sha1, upload_git_object, vastrum_get_head_commit,
    },
};
use gix_actor::Signature;
use gix_date::Time;
use gix_hash::{Kind, ObjectId};
use gix_object::{
    Commit, Object, Tree,
    bstr::BString,
    tree::{Entry, EntryKind, EntryMode},
};
use std::collections::HashSet;
use vastrum_rpc_client::SentTxBehavior;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::native::upload::push_to_repo;
    use crate::testing::test_helpers::{TestContext, TestRepoBuilder};
    use serial_test::serial;
    use vastrum_rpc_client::SentTxBehavior;

    #[tokio::test]
    #[serial]
    async fn test_merge_operations() {
        let ctx = TestContext::new().await;

        // Base repo
        let base = TestRepoBuilder::new().file("file.txt", b"base").build();
        ctx.contract.create_repository("base", "").await.await_confirmation().await;
        push_to_repo(base.path_str(), "base", &ctx.contract, None).await.unwrap();

        // Branch 1: same base + new file (fast-forward)
        let b1 = TestRepoBuilder::new().file("file.txt", b"base").build();
        let b1_commit = b1.add_commit(&[("file.txt", b"base"), ("new.txt", b"new")]);
        ctx.contract.create_repository("b1", "").await.await_confirmation().await;
        push_to_repo(b1.path_str(), "b1", &ctx.contract, None).await.unwrap();

        // Branch 2: same base + different file (3-way merge)
        let b2 = TestRepoBuilder::new().file("file.txt", b"base").build();
        let b2_commit = b2.add_commit(&[("file.txt", b"base"), ("other.txt", b"other")]);
        ctx.contract.create_repository("b2", "").await.await_confirmation().await;
        push_to_repo(b2.path_str(), "b2", &ctx.contract, None).await.unwrap();

        // Fast-forward: result should be b1's HEAD
        let result = merge_repos("base", "b1", &ctx.contract, MergeMode::Live).await.unwrap();
        assert_eq!(result, MergeResult::FastForward(b1_commit));

        // Already up-to-date
        let result = merge_repos("base", "b1", &ctx.contract, MergeMode::Live).await.unwrap();
        assert_eq!(result, MergeResult::AlreadyUpToDate);

        // 3-way merge: verify merge commit has correct parents
        let result = merge_repos("base", "b2", &ctx.contract, MergeMode::Live).await.unwrap();
        if let MergeResult::Merged(merge_commit_id) = result {
            let git_ctx = GitContext::from_contract(&ctx.contract).await;
            let merge_commit = git_ctx.read_commit(merge_commit_id).await.unwrap();
            // After fast-forward, base HEAD is now b1_commit
            assert_eq!(merge_commit.parents.len(), 2);
            assert_eq!(merge_commit.parents[0], b1_commit);
            assert_eq!(merge_commit.parents[1], b2_commit);
        } else {
            panic!("Expected Merged, got {:?}", result);
        }

        // Conflict test: create fresh repos where both sides modify file.txt differently
        let cbase = TestRepoBuilder::new().file("file.txt", b"original").build();
        ctx.contract.create_repository("cbase", "").await.await_confirmation().await;
        push_to_repo(cbase.path_str(), "cbase", &ctx.contract, None).await.unwrap();

        // Branch modifies file one way
        let cb1 = TestRepoBuilder::new().file("file.txt", b"original").build();
        let cb1_commit = cb1.add_commit(&[("file.txt", b"change1")]);
        let cb1_blob = cb1.blob_id("file.txt");
        ctx.contract.create_repository("cb1", "").await.await_confirmation().await;
        push_to_repo(cb1.path_str(), "cb1", &ctx.contract, None).await.unwrap();

        // Branch modifies file different way
        let cb2 = TestRepoBuilder::new().file("file.txt", b"original").build();
        cb2.add_commit(&[("file.txt", b"change2")]);
        let cb2_blob = cb2.blob_id("file.txt");
        ctx.contract.create_repository("cb2", "").await.await_confirmation().await;
        push_to_repo(cb2.path_str(), "cb2", &ctx.contract, None).await.unwrap();

        // First merge succeeds (fast-forward)
        let result = merge_repos("cbase", "cb1", &ctx.contract, MergeMode::Live).await.unwrap();
        assert_eq!(result, MergeResult::FastForward(cb1_commit));

        // Second merge should conflict - verify conflict details
        let result = merge_repos("cbase", "cb2", &ctx.contract, MergeMode::Live).await.unwrap();
        if let MergeResult::Conflict(conflicts) = result {
            assert_eq!(conflicts.len(), 1);
            assert_eq!(conflicts[0].path, "file.txt");
            assert_eq!(conflicts[0].ours_oid, cb1_blob);
            assert_eq!(conflicts[0].theirs_oid, cb2_blob);
        } else {
            panic!("Expected Conflict, got {:?}", result);
        }
    }

    #[tokio::test]
    #[serial]
    async fn test_nested_directory_merge() {
        let ctx = TestContext::new().await;

        // Base with nested structure
        let nbase = TestRepoBuilder::new()
            .file("src/main.rs", b"main")
            .file("docs/readme.md", b"readme")
            .build();
        ctx.contract.create_repository("nbase", "").await.await_confirmation().await;
        push_to_repo(nbase.path_str(), "nbase", &ctx.contract, None).await.unwrap();

        // Branch adds src/lib.rs
        let nb1 = TestRepoBuilder::new()
            .file("src/main.rs", b"main")
            .file("docs/readme.md", b"readme")
            .build();
        let nb1_commit = nb1.add_commit(&[
            ("src/main.rs", b"main"),
            ("src/lib.rs", b"lib"),
            ("docs/readme.md", b"readme"),
        ]);
        ctx.contract.create_repository("nb1", "").await.await_confirmation().await;
        push_to_repo(nb1.path_str(), "nb1", &ctx.contract, None).await.unwrap();

        // Branch adds docs/api.md
        let nb2 = TestRepoBuilder::new()
            .file("src/main.rs", b"main")
            .file("docs/readme.md", b"readme")
            .build();
        let nb2_commit = nb2.add_commit(&[
            ("src/main.rs", b"main"),
            ("docs/readme.md", b"readme"),
            ("docs/api.md", b"api"),
        ]);
        ctx.contract.create_repository("nb2", "").await.await_confirmation().await;
        push_to_repo(nb2.path_str(), "nb2", &ctx.contract, None).await.unwrap();

        // Fast-forward
        let result = merge_repos("nbase", "nb1", &ctx.contract, MergeMode::Live).await.unwrap();
        assert_eq!(result, MergeResult::FastForward(nb1_commit));

        // 3-way merge: verify parents
        let result = merge_repos("nbase", "nb2", &ctx.contract, MergeMode::Live).await.unwrap();
        if let MergeResult::Merged(merge_commit_id) = result {
            let git_ctx = GitContext::from_contract(&ctx.contract).await;
            let merge_commit = git_ctx.read_commit(merge_commit_id).await.unwrap();
            assert_eq!(merge_commit.parents.len(), 2);
            assert_eq!(merge_commit.parents[0], nb1_commit);
            assert_eq!(merge_commit.parents[1], nb2_commit);
        } else {
            panic!("Expected Merged, got {:?}", result);
        }
    }

    #[tokio::test]
    #[serial]
    async fn test_delete_modify_conflict() {
        let ctx = TestContext::new().await;

        let dm = TestRepoBuilder::new().file("file.txt", b"content").build();
        let dm_blob = dm.blob_id("file.txt");
        ctx.contract.create_repository("dm", "").await.await_confirmation().await;
        push_to_repo(dm.path_str(), "dm", &ctx.contract, None).await.unwrap();

        // Branch deletes file
        let dm1 = TestRepoBuilder::new().file("file.txt", b"content").build();
        let dm1_commit = dm1.add_commit(&[]);
        ctx.contract.create_repository("dm1", "").await.await_confirmation().await;
        push_to_repo(dm1.path_str(), "dm1", &ctx.contract, None).await.unwrap();

        // Branch modifies file
        let dm2 = TestRepoBuilder::new().file("file.txt", b"content").build();
        dm2.add_commit(&[("file.txt", b"modified")]);
        let dm2_blob = dm2.blob_id("file.txt");
        ctx.contract.create_repository("dm2", "").await.await_confirmation().await;
        push_to_repo(dm2.path_str(), "dm2", &ctx.contract, None).await.unwrap();

        // Fast-forward (delete branch)
        let result = merge_repos("dm", "dm1", &ctx.contract, MergeMode::Live).await.unwrap();
        assert_eq!(result, MergeResult::FastForward(dm1_commit));

        // Conflict: delete vs modify
        let result = merge_repos("dm", "dm2", &ctx.contract, MergeMode::Live).await.unwrap();
        if let MergeResult::Conflict(conflicts) = result {
            assert_eq!(conflicts.len(), 1);
            assert_eq!(conflicts[0].path, "file.txt");
            // ours deleted file, so ours_oid falls back to base blob
            assert_eq!(conflicts[0].ours_oid, dm_blob);
            assert_eq!(conflicts[0].theirs_oid, dm2_blob);
        } else {
            panic!("Expected Conflict, got {:?}", result);
        }
    }
}
