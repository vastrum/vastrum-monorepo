pub async fn clone_repo(
    repo_name: &str,
    target_path: impl AsRef<Path>,
    contract: &ContractAbiClient,
    progress: Option<&ProgressBar>,
) -> Result<()> {
    let target_path = target_path.as_ref();
    let head = vastrum_get_head_commit(repo_name, contract).await?;

    let mut new_repo = ThreadSafeRepository::init_opts(
        target_path,
        Kind::WithWorktree,
        create::Options::default(),
        open::Options::default(),
    )?
    .to_thread_local();

    let _ = new_repo.committer_or_set_generic_fallback();

    download_commits(&new_repo, head, contract, progress).await?;

    new_repo.edit_reference(RefEdit {
        change: Change::Update {
            log: LogChange {
                mode: RefLog::AndReference,
                force_create_reflog: false,
                message: "clone".into(),
            },
            expected: PreviousValue::Any,
            new: Target::Object(head),
        },
        name: "HEAD".try_into().unwrap(),
        deref: false,
    })?;

    checkout_head_to_workdir(&new_repo, head, target_path)?;

    return Ok(());
}

//recursively download all commits
pub async fn download_commits(
    repository: &Repository,
    oid: ObjectId,
    contract: &ContractAbiClient,
    progress: Option<&ProgressBar>,
) -> Result<()> {
    let parent_ids = download_commit(oid, repository, contract, progress).await?;
    for parent in parent_ids {
        Box::pin(download_commits(repository, parent, contract, progress)).await?;
    }
    return Ok(());
}

pub async fn download_commit(
    oid: ObjectId,
    repo: &Repository,
    contract: &ContractAbiClient,
    progress: Option<&ProgressBar>,
) -> Result<Vec<ObjectId>> {
    let commit_already_downloaded = repo.find_object(oid).is_ok();
    if commit_already_downloaded {
        return Ok(vec![]);
    }

    let ctx = GitContext::from_contract(contract).await;
    let commit = ctx.read_commit(oid).await?;
    let parents = commit.parents.to_vec();

    download_trees(commit.tree, repo, contract, progress).await?;

    repo.write_object(commit)?;

    if let Some(progress) = progress {
        progress.inc(1);
    }

    return Ok(parents);
}

pub async fn download_trees(
    tree_id: ObjectId,
    repo: &Repository,
    contract: &ContractAbiClient,
    progress: Option<&ProgressBar>,
) -> Result<()> {
    let ctx = GitContext::from_contract(contract).await;
    let tree = ctx.read_tree(tree_id).await?;
    download_tree(tree, tree_id, repo, &ctx, progress).await
}

pub async fn download_tree(
    tree: Tree,
    tree_id: ObjectId,
    repo: &Repository,
    ctx: &GitContext,
    progress: Option<&ProgressBar>,
) -> Result<()> {
    let tree_already_downloaded = repo.find_tree(tree_id).is_ok();
    if tree_already_downloaded {
        return Ok(());
    }
    repo.write_object(&tree)?;
    if let Some(progress) = progress {
        progress.inc(1);
    }

    for entry in tree.entries {
        let kind = entry.mode.kind();

        if kind == EntryKind::Tree {
            let entry_not_yet_downloaded = repo.find_tree(entry.oid).is_err();
            if entry_not_yet_downloaded {
                let tree = ctx.read_tree(entry.oid).await?;
                Box::pin(download_tree(tree, entry.oid, repo, ctx, progress)).await?;
            }
        } else if kind == EntryKind::Commit {
            return Err(VastrumGitError::SubmodulesNotSupported);
        } else {
            let blob_not_yet_downloaded = repo.find_blob(entry.oid).is_err();
            if blob_not_yet_downloaded {
                let blob = ctx.blob_read(entry.oid).await?;
                repo.write_object(blob)?;
                if let Some(progress) = progress {
                    progress.inc(1);
                }
            }
        }
    }
    return Ok(());
}

fn checkout_head_to_workdir(repo: &Repository, head: ObjectId, work_dir: &Path) -> Result<()> {
    let commit_obj = repo.find_object(head)?;
    let commit_ref = CommitRef::from_bytes(&commit_obj.data)?;
    let tree_id = commit_ref.tree();

    let Ok(mut index) = index::State::from_tree(&tree_id, &repo.objects, Default::default()) else {
        return Err(VastrumGitError::Checkout("failed to build index from tree".into()));
    };

    let Ok(_) = worktree::state::checkout(
        &mut index,
        work_dir,
        repo.objects.clone(),
        &Discard,
        &Discard,
        &AtomicBool::new(false),
        worktree::state::checkout::Options {
            destination_is_initially_empty: true,
            ..Default::default()
        },
    ) else {
        return Err(VastrumGitError::Checkout("failed to checkout working tree".into()));
    };

    let mut index_file = index::File::from_state(index, repo.index_path());
    index_file.write(Default::default()).unwrap();

    return Ok(());
}

use crate::{
    ContractAbiClient,
    error::{Result, VastrumGitError},
    universal::utils::{GitContext, vastrum_get_head_commit},
};
use gix::{
    ObjectId, Repository, ThreadSafeRepository, create,
    create::Kind,
    index,
    objs::{Tree, tree::EntryKind},
    open,
    progress::Discard,
    refs::{
        Target,
        transaction::{Change, LogChange, PreviousValue, RefEdit, RefLog},
    },
    worktree,
};
use gix_object::CommitRef;
use indicatif::ProgressBar;
use std::path::Path;
use std::sync::atomic::AtomicBool;
