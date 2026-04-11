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
use vastrum_git_lib::universal::utils::{GitContext, sha1_to_oid};

/// Create and populate a bare repo from on-chain state.
/// Downloads all branches and writes each as `refs/heads/{branch}`.
/// Sets HEAD symbolic ref to the default branch.
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

    let ctx = GitContext::new(state.git_object_store);

    // Download each branch and write its ref
    for (branch_name, hash) in &repo_info.branches {
        let oid = sha1_to_oid(hash);
        vastrum_git_lib::native::clone::download_commits(&repo, oid, &ctx, None).await?;

        let ref_name = format!("refs/heads/{}", branch_name);
        repo.edit_reference(RefEdit {
            change: Change::Update {
                log: LogChange {
                    mode: RefLog::AndReference,
                    force_create_reflog: false,
                    message: "materialize from chain".into(),
                },
                expected: PreviousValue::Any,
                new: Target::Object(oid),
            },
            name: ref_name.try_into().unwrap(),
            deref: false,
        })?;
    }

    // Set HEAD to default branch (even if the branch doesn't exist yet — unborn HEAD
    // is fine for git-receive-pack, it creates the branch on first push).
    let head_target = format!("refs/heads/{}", repo_info.default_branch);
    repo.edit_reference(RefEdit {
        change: Change::Update {
            log: LogChange {
                mode: RefLog::AndReference,
                force_create_reflog: false,
                message: "set HEAD to default branch".into(),
            },
            expected: PreviousValue::Any,
            new: Target::Symbolic(head_target.try_into().unwrap()),
        },
        name: "HEAD".try_into().unwrap(),
        deref: false,
    })?;

    Ok(())
}
