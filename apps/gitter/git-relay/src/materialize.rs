use anyhow::{Context, Result};
use gix::{
    ThreadSafeRepository, create,
    create::Kind,
    open,
    refs::{
        Target,
        transaction::{Change, LogChange, PreviousValue, RefEdit, RefLog},
    },
};
use std::path::Path;
use vastrum_git_lib::ContractAbiClient;
use vastrum_git_lib::universal::utils::{sha1_to_oid, GitContext};

/// Create and populate a bare repo from on-chain state.
/// If the repo has no head commit on chain, creates an empty bare repo.
pub async fn materialize_bare_repo(
    bare_path: &Path,
    repo_name: &str,
    contract: &ContractAbiClient,
) -> Result<()> {
    let state = contract.state().await;
    let repo_info = state
        .repo_store
        .get(&repo_name.to_string())
        .await
        .with_context(|| format!("repo '{}' not found on-chain", repo_name))?;

    // Create bare repo
    let repo = ThreadSafeRepository::init_opts(
        bare_path,
        Kind::Bare,
        create::Options::default(),
        open::Options::default(),
    )?
    .to_thread_local();

    // If no head commit on chain, nothing to download
    let Some(head_hash) = repo_info.head_commit_hash else {
        return Ok(());
    };
    let head = sha1_to_oid(&head_hash);
    let ctx = GitContext::new(state.git_object_store);

    vastrum_git_lib::native::clone::download_commits(&repo, head, &ctx, None).await?;

    // Point refs/heads/main at the on-chain HEAD
    repo.edit_reference(RefEdit {
        change: Change::Update {
            log: LogChange {
                mode: RefLog::AndReference,
                force_create_reflog: false,
                message: "materialize from chain".into(),
            },
            expected: PreviousValue::Any,
            new: Target::Object(head),
        },
        name: "refs/heads/main".try_into().unwrap(),
        deref: false,
    })?;

    Ok(())
}
