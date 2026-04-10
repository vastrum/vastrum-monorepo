use crate::repo_cache::RepoCache;
use std::sync::Arc;
use std::time::Duration;

/// Runs the background chain sync loop on the current async runtime.
/// Must be called within a tokio `LocalSet` or on a single-threaded runtime
/// because gix::Repository is !Send.
pub async fn run_sync_loop(cache: Arc<RepoCache>, interval_secs: u64) {
    let interval = Duration::from_secs(interval_secs);
    tracing::info!(interval_secs, "background sync started");

    loop {
        tokio::time::sleep(interval).await;

        let repos = match cache.list_cached_repos() {
            Ok(repos) => repos,
            Err(e) => {
                tracing::warn!(error = %e, "failed to list cached repos");
                continue;
            }
        };

        for repo_name in repos {
            if let Err(e) = cache.ensure_fresh(&repo_name).await {
                tracing::warn!(repo = repo_name, error = %e, "background sync failed");
            }
        }
    }
}
