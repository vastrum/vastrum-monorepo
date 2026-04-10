use anyhow::{Context, Result};
use std::path::Path;
use vastrum_git_lib::ContractAbiClient;
use vastrum_git_lib::universal::utils::{oid_to_sha1, sha1_to_oid};
use vastrum_rpc_client::SentTxBehavior;

/// After git receive-pack completes, sync new objects from the local bare repo to chain.
pub async fn sync_to_chain(
    repo_path: &Path,
    repo_name: &str,
    relay_contract: &ContractAbiClient,
) -> Result<PushSyncResult> {
    // Get on-chain HEAD
    let state = relay_contract.state().await;
    let repo_info = state
        .repo_store
        .get(&repo_name.to_string())
        .await
        .with_context(|| format!("repo '{}' not found on-chain", repo_name))?;

    let on_chain_head = repo_info.head_commit_hash.as_ref().map(|h| sha1_to_oid(h));

    // Collect objects in a blocking task (gix::Repository is not Send)
    let repo_path_owned = repo_path.to_path_buf();
    let on_chain_head_clone = on_chain_head;
    let (local_head, objects) = tokio::task::spawn_blocking(move || {
        collect_objects_blocking(&repo_path_owned, on_chain_head_clone)
    })
    .await??;

    // Check if already in sync
    if on_chain_head == Some(local_head) {
        return Ok(PushSyncResult::AlreadyUpToDate);
    }

    // Upload objects (unauthenticated — anyone can upload git objects)
    let uploaded =
        vastrum_git_lib::native::upload::upload_objects_concurrent(&objects, relay_contract, None)
            .await
            .context("failed to upload objects to chain")?;

    // Update on-chain HEAD via relay-authenticated call
    let hash = oid_to_sha1(local_head);
    relay_contract
        .relay_set_head_commit(repo_name, hash)
        .await
        .await_confirmation()
        .await;

    tracing::info!(
        repo = repo_name,
        objects = uploaded,
        head = %local_head,
        "pushed to chain via relay"
    );

    Ok(PushSyncResult::Pushed {
        objects_uploaded: uploaded,
    })
}

/// Collect git objects in a blocking context (gix::Repository is !Send).
fn collect_objects_blocking(
    repo_path: &Path,
    on_chain_head: Option<gix_hash::ObjectId>,
) -> Result<(gix_hash::ObjectId, Vec<gix_object::Object>)> {
    let repo = gix::open(repo_path).context("failed to open bare repo")?;
    let local_head = repo.head_id().context("bare repo has no HEAD")?.detach();
    let objects =
        vastrum_git_lib::native::upload::collect_all_objects(&repo, local_head, on_chain_head)
            .context("failed to collect objects for upload")?;
    Ok((local_head, objects))
}

pub enum PushSyncResult {
    Pushed { objects_uploaded: usize },
    AlreadyUpToDate,
}
