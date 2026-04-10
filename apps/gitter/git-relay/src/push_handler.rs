use anyhow::{Context, Result};
use std::path::Path;
use vastrum_git_lib::ContractAbiClient;
use vastrum_git_lib::universal::utils::{oid_to_sha1, sha1_to_oid};
use vastrum_rpc_client::SentTxBehavior;

/// Collect new git objects that need uploading to chain.
/// Returns (local_head, objects, on_chain_head).
pub async fn collect_new_objects(
    repo_path: &Path,
    repo_name: &str,
    relay_contract: &ContractAbiClient,
) -> Result<CollectedObjects> {
    let state = relay_contract.state().await;
    let repo_info = state
        .repo_store
        .get(&repo_name.to_string())
        .await
        .with_context(|| format!("repo '{}' not found on-chain", repo_name))?;

    let on_chain_head = repo_info.head_commit_hash.as_ref().map(|h| sha1_to_oid(h));

    let repo_path_owned = repo_path.to_path_buf();
    let on_chain_head_clone = on_chain_head;
    let (local_head, objects) = tokio::task::spawn_blocking(move || {
        let repo = gix::open(&repo_path_owned).context("failed to open bare repo")?;
        let local_head = repo.head_id().context("bare repo has no HEAD")?.detach();
        let objects = vastrum_git_lib::native::upload::collect_all_objects(
            &repo,
            local_head,
            on_chain_head_clone,
        )
        .context("failed to collect objects")?;
        Ok::<_, anyhow::Error>((local_head, objects))
    })
    .await??;

    if on_chain_head == Some(local_head) {
        return Ok(CollectedObjects::AlreadyUpToDate);
    }

    Ok(CollectedObjects::New {
        local_head,
        objects,
    })
}

/// Upload collected objects to chain.
pub async fn upload_objects(
    objects: &[gix_object::Object],
    relay_contract: &ContractAbiClient,
) -> Result<usize> {
    vastrum_git_lib::native::upload::upload_objects_concurrent(objects, relay_contract, None)
        .await
        .context("failed to upload objects to chain")
}

/// Update on-chain HEAD via relay-authenticated call and verify it took effect.
pub async fn update_and_verify_head(
    repo_name: &str,
    local_head: gix_hash::ObjectId,
    relay_contract: &ContractAbiClient,
) -> Result<()> {
    let hash = oid_to_sha1(local_head);
    relay_contract
        .relay_set_head_commit(repo_name, hash)
        .await
        .await_confirmation()
        .await;

    // Verify
    let state = relay_contract.state().await;
    let updated_repo = state
        .repo_store
        .get(&repo_name.to_string())
        .await
        .with_context(|| "repo disappeared after push")?;
    let updated_head = updated_repo.head_commit_hash.as_ref().map(|h| sha1_to_oid(h));
    if updated_head != Some(local_head) {
        anyhow::bail!(
            "relay_set_head_commit failed: on-chain HEAD is {:?}, expected {}",
            updated_head,
            local_head
        );
    }
    Ok(())
}

pub enum CollectedObjects {
    AlreadyUpToDate,
    New {
        local_head: gix_hash::ObjectId,
        objects: Vec<gix_object::Object>,
    },
}
