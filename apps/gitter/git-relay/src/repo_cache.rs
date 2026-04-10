use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::sync::Mutex;
use vastrum_git_lib::ContractAbiClient;

/// Manages local bare git repositories as a cache of on-chain state.
///
/// All operations that touch gix are run in `spawn_blocking` to avoid
/// Send issues (gix::Repository is !Send).
pub struct RepoCache {
    data_dir: PathBuf,
    contract: ContractAbiClient,
    /// Per-repo locks to prevent concurrent sync operations on the same repo.
    locks: Mutex<HashMap<String, std::sync::Arc<tokio::sync::Mutex<()>>>>,
}

impl RepoCache {
    pub fn new(data_dir: PathBuf, contract: ContractAbiClient) -> Self {
        Self {
            data_dir,
            contract,
            locks: Mutex::new(HashMap::new()),
        }
    }

    /// Returns the path to the bare repo on disk. Creates if needed, syncs if stale.
    pub async fn ensure_fresh(&self, repo_name: &str) -> Result<PathBuf> {
        let lock = {
            let mut locks = self.locks.lock().await;
            locks
                .entry(repo_name.to_string())
                .or_insert_with(|| std::sync::Arc::new(tokio::sync::Mutex::new(())))
                .clone()
        };
        let _guard = lock.lock().await;

        let repo_path = self.repo_path(repo_name);

        if !repo_path.exists() {
            self.clone_from_chain(repo_name, &repo_path).await?;
        } else {
            self.sync_from_chain(repo_name, &repo_path).await?;
        }

        Ok(repo_path)
    }

    /// Ensures the bare repo exists on disk without syncing (for receive-pack).
    pub async fn ensure_exists(&self, repo_name: &str) -> Result<PathBuf> {
        let repo_path = self.repo_path(repo_name);
        if !repo_path.exists() {
            self.clone_from_chain(repo_name, &repo_path).await?;
        }
        Ok(repo_path)
    }

    pub fn repo_path(&self, repo_name: &str) -> PathBuf {
        self.data_dir.join(format!("{}.git", repo_name))
    }

    pub fn contract(&self) -> &ContractAbiClient {
        &self.contract
    }

    /// Clone a repo from on-chain state into a new local bare repo.
    ///
    /// Uses `spawn_blocking` + nested runtime to handle the gix !Send issue.
    async fn clone_from_chain(&self, repo_name: &str, repo_path: &Path) -> Result<()> {
        let state = self.contract.state().await;
        let repo_info = state
            .repo_store
            .get(&repo_name.to_string())
            .await
            .with_context(|| format!("repo '{}' not found on-chain", repo_name))?;

        // Create bare repo
        std::fs::create_dir_all(repo_path)?;
        let output = tokio::process::Command::new("git")
            .args(["init", "--bare"])
            .arg(repo_path)
            .output()
            .await?;
        if !output.status.success() {
            anyhow::bail!(
                "git init --bare failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }

        if let Some(head_hash) = repo_info.head_commit_hash {
            // clone_repo uses gix internally which is !Send.
            // Run it on a blocking thread with its own runtime handle.
            let contract = ContractAbiClient::new(self.contract.site_id());
            let repo_name_owned = repo_name.to_string();
            let repo_path_owned = repo_path.to_path_buf();
            let handle = tokio::runtime::Handle::current();

            tokio::task::spawn_blocking(move || {
                let tmp_dir = tempfile::tempdir().unwrap();
                let tmp_path = tmp_dir.path().to_string_lossy().to_string();

                handle.block_on(async {
                    vastrum_git_lib::native::clone::clone_repo(
                        &repo_name_owned,
                        &tmp_path,
                        &contract,
                        None,
                    )
                    .await
                })?;

                // Copy objects from the cloned repo into the bare repo
                let tmp_objects = tmp_dir.path().join(".git/objects");
                let bare_objects = repo_path_owned.join("objects");
                copy_git_objects(&tmp_objects, &bare_objects)?;

                // Update HEAD ref in bare repo
                let head_hex = hex::encode(head_hash.0);
                std::fs::create_dir_all(repo_path_owned.join("refs/heads"))?;
                std::fs::write(
                    repo_path_owned.join("refs/heads/main"),
                    format!("{}\n", head_hex),
                )?;
                std::fs::write(repo_path_owned.join("HEAD"), "ref: refs/heads/main\n")?;

                Ok::<_, anyhow::Error>(())
            })
            .await??;
        }

        tracing::info!(repo = repo_name, "cloned from chain into cache");
        Ok(())
    }

    /// Sync an existing bare repo with on-chain state (pull new objects).
    async fn sync_from_chain(&self, repo_name: &str, repo_path: &Path) -> Result<()> {
        let state = self.contract.state().await;
        let repo_info = match state.repo_store.get(&repo_name.to_string()).await {
            Some(info) => info,
            None => return Ok(()),
        };

        let on_chain_head = match repo_info.head_commit_hash {
            Some(h) => h,
            None => return Ok(()),
        };

        // Check if local HEAD matches on-chain HEAD
        let local_head = get_bare_repo_head(repo_path);
        let on_chain_hex = hex::encode(on_chain_head.0);

        if local_head.as_deref() == Some(on_chain_hex.as_str()) {
            return Ok(()); // Already in sync
        }

        tracing::info!(
            repo = repo_name,
            local = ?local_head,
            chain = on_chain_hex,
            "cache stale, syncing from chain"
        );

        // Run clone in a blocking thread (gix is !Send)
        let contract = ContractAbiClient::new(self.contract.site_id());
        let repo_name_owned = repo_name.to_string();
        let repo_path_owned = repo_path.to_path_buf();
        let handle = tokio::runtime::Handle::current();

        tokio::task::spawn_blocking(move || {
            let tmp_dir = tempfile::tempdir().unwrap();
            let tmp_path = tmp_dir.path().to_string_lossy().to_string();

            handle.block_on(async {
                vastrum_git_lib::native::clone::clone_repo(
                    &repo_name_owned,
                    &tmp_path,
                    &contract,
                    None,
                )
                .await
            })?;

            let tmp_objects = tmp_dir.path().join(".git/objects");
            let bare_objects = repo_path_owned.join("objects");
            copy_git_objects(&tmp_objects, &bare_objects)?;

            // Update HEAD ref
            let on_chain_hex = hex::encode(on_chain_head.0);
            std::fs::create_dir_all(repo_path_owned.join("refs/heads"))?;
            std::fs::write(
                repo_path_owned.join("refs/heads/main"),
                format!("{}\n", on_chain_hex),
            )?;

            Ok::<_, anyhow::Error>(())
        })
        .await??;

        tracing::info!(repo = repo_name, head = on_chain_hex, "cache synced");
        Ok(())
    }

    /// Get all cached repo names (for background sync).
    pub fn list_cached_repos(&self) -> Result<Vec<String>> {
        let mut repos = Vec::new();
        if !self.data_dir.exists() {
            return Ok(repos);
        }
        for entry in std::fs::read_dir(&self.data_dir)? {
            let entry = entry?;
            let name = entry.file_name().to_string_lossy().to_string();
            if name.ends_with(".git") {
                repos.push(name.trim_end_matches(".git").to_string());
            }
        }
        Ok(repos)
    }
}

/// Read the HEAD commit hash from a bare repo's refs/heads/main.
fn get_bare_repo_head(repo_path: &Path) -> Option<String> {
    let ref_path = repo_path.join("refs/heads/main");
    std::fs::read_to_string(ref_path)
        .ok()
        .map(|s| s.trim().to_string())
}

/// Copy git objects from one objects/ directory to another (loose + pack).
fn copy_git_objects(src: &Path, dst: &Path) -> Result<()> {
    if !src.exists() {
        return Ok(());
    }

    for entry in walkdir::WalkDir::new(src).min_depth(1) {
        let entry = entry?;
        let rel = entry.path().strip_prefix(src)?;
        let dst_path = dst.join(rel);

        if entry.file_type().is_dir() {
            std::fs::create_dir_all(&dst_path)?;
        } else if entry.file_type().is_file() {
            if let Some(parent) = dst_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::copy(entry.path(), &dst_path)?;
        }
    }
    Ok(())
}
