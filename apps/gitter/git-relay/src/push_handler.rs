use anyhow::{Context, Result};
use std::collections::BTreeMap;
use std::path::Path;
use vastrum_git_lib::ContractAbiClient;
use vastrum_git_lib::universal::utils::{oid_to_sha1, sha1_to_oid};
use vastrum_rpc_client::SentTxBehavior;

/// Per-branch sync work: which branches changed, which were deleted, and what
/// objects need uploading.
pub struct PushPlan {
    /// Branches to update on-chain: (branch_name, local_head_oid)
    pub updates: Vec<(String, gix_hash::ObjectId)>,
    /// Branches to delete on-chain
    pub deletes: Vec<String>,
    /// Objects to upload (deduplicated across all updated branches)
    pub objects: Vec<gix_object::Object>,
}

/// Diff local refs against on-chain state and collect new objects for all changed branches.
pub async fn collect_push_plan(
    repo_path: &Path,
    repo_name: &str,
    relay_contract: &ContractAbiClient,
) -> Result<PushPlan> {
    let state = relay_contract.state().await;
    let repo_info = state
        .repo_store
        .get(&repo_name.to_string())
        .await
        .with_context(|| format!("repo '{}' not found on-chain", repo_name))?;

    // Snapshot on-chain branches
    let on_chain_branches: BTreeMap<String, gix_hash::ObjectId> = repo_info
        .branches
        .iter()
        .map(|(name, hash)| (name.clone(), sha1_to_oid(hash)))
        .collect();

    let repo_path_owned = repo_path.to_path_buf();
    let plan = tokio::task::spawn_blocking(move || -> Result<PushPlan> {
        let repo = gix::open(&repo_path_owned).context("failed to open bare repo")?;

        // Walk local refs/heads/*
        let mut local_branches: BTreeMap<String, gix_hash::ObjectId> = BTreeMap::new();
        let platform = repo.references().context("failed to get references")?;
        let prefixed = platform
            .prefixed("refs/heads/")
            .map_err(|e| anyhow::anyhow!("failed to list local branches: {}", e))?;
        for reference in prefixed {
            let reference =
                reference.map_err(|e| anyhow::anyhow!("failed to read reference: {}", e))?;
            let name = reference.name().as_bstr().to_string();
            let Some(branch_name) = name.strip_prefix("refs/heads/") else {
                continue;
            };
            let branch_name: String = branch_name.to_string();
            if let gix::refs::TargetRef::Object(oid) = reference.target() {
                local_branches.insert(branch_name, oid.to_owned());
            }
        }

        // Determine updates and deletes
        let mut updates = Vec::new();
        let mut deletes = Vec::new();
        let mut all_objects = Vec::new();
        let mut seen_oids = std::collections::HashSet::new();

        for (branch, local_oid) in &local_branches {
            let on_chain_oid = on_chain_branches.get(branch).copied();
            if on_chain_oid == Some(*local_oid) {
                continue; // Already in sync
            }
            // Collect objects from local_oid, stopping at on_chain_oid if ancestor
            let objects = vastrum_git_lib::native::upload::collect_all_objects(
                &repo,
                *local_oid,
                on_chain_oid,
            )
            .with_context(|| format!("failed to collect objects for branch {}", branch))?;
            // Dedupe objects across branches (share common history)
            for obj in objects {
                let hash = vastrum_git_lib::universal::utils::calculate_object_hash(&obj);
                if seen_oids.insert(hash) {
                    all_objects.push(obj);
                }
            }
            updates.push((branch.clone(), *local_oid));
        }

        for branch in on_chain_branches.keys() {
            if !local_branches.contains_key(branch) {
                deletes.push(branch.clone());
            }
        }

        Ok(PushPlan {
            updates,
            deletes,
            objects: all_objects,
        })
    })
    .await??;

    Ok(plan)
}

/// Upload collected objects to chain.
pub async fn upload_objects(
    objects: &[gix_object::Object],
    relay_contract: &ContractAbiClient,
    uploaded_counter: Option<&std::sync::atomic::AtomicUsize>,
) -> Result<usize> {
    vastrum_git_lib::native::upload::upload_objects_concurrent(
        objects,
        relay_contract,
        None,
        uploaded_counter,
    )
    .await
    .context("failed to upload objects to chain")
}

/// Apply branch updates and deletions on-chain.
pub async fn apply_branch_updates(
    repo_name: &str,
    plan: &PushPlan,
    relay_contract: &ContractAbiClient,
) -> Result<()> {
    for (branch, oid) in &plan.updates {
        let hash = oid_to_sha1(*oid);
        relay_contract
            .relay_set_branch_head(repo_name, branch.clone(), hash)
            .await
            .await_confirmation()
            .await;
    }
    for branch in &plan.deletes {
        relay_contract
            .relay_delete_branch(repo_name, branch.clone())
            .await
            .await_confirmation()
            .await;
    }

    // Verify
    let state = relay_contract.state().await;
    let updated_repo = state
        .repo_store
        .get(&repo_name.to_string())
        .await
        .with_context(|| "repo disappeared after push")?;

    for (branch, oid) in &plan.updates {
        let on_chain = updated_repo.branches.get(branch).map(|h| sha1_to_oid(h));
        if on_chain != Some(*oid) {
            anyhow::bail!(
                "relay_set_branch_head failed: branch {} is {:?}, expected {}",
                branch,
                on_chain,
                oid
            );
        }
    }
    for branch in &plan.deletes {
        if updated_repo.branches.contains_key(branch) {
            anyhow::bail!("relay_delete_branch failed: branch {} still present", branch);
        }
    }

    Ok(())
}
