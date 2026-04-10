mod chain_sync;
mod config;
mod http_server;
mod push_handler;
mod repo_cache;
mod ssh_server;

use anyhow::{Context, Result};
use config::Config;
use repo_cache::RepoCache;
use ssh_server::SshServer;
use std::sync::Arc;
use vastrum_git_lib::ContractAbiClient;
use vastrum_native_lib::NativeHttpClient;
use vastrum_shared_types::crypto::ed25519;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,vastrum_git_relay=debug".parse().unwrap()),
        )
        .init();

    let config_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "relay.toml".to_string());

    let config = Config::load(&config_path)
        .with_context(|| format!("failed to load config from '{}'", config_path))?;

    // Read relay private key from file
    let relay_private_key_str = std::fs::read_to_string(&config.chain.relay_key_path)
        .with_context(|| {
            format!(
                "failed to read relay key from '{}'",
                config.chain.relay_key_path.display()
            )
        })?;
    let relay_private_key =
        ed25519::PrivateKey::try_from_string(relay_private_key_str.trim().to_string())
            .with_context(|| "invalid relay private key")?;

    // Resolve gitter site ID
    let http = NativeHttpClient::new();
    let site_id = http
        .resolve_domain(config.chain.gitter_domain.clone())
        .await?
        .with_context(|| {
            format!(
                "could not resolve domain: {}",
                config.chain.gitter_domain
            )
        })?;

    tracing::info!(
        site_id = %site_id,
        domain = config.chain.gitter_domain,
        "resolved gitter contract"
    );

    // Create contract client authenticated with relay key
    let site_key =
        vastrum_shared_types::crypto::site_key::derive_site_key(&relay_private_key, site_id);
    let contract = ContractAbiClient::new(site_id).with_account_key(site_key);

    // Create data directory
    std::fs::create_dir_all(&config.server.data_dir)?;

    // Shared repo cache
    let cache = Arc::new(RepoCache::new(config.server.data_dir.clone(), contract));

    // Start background sync on a dedicated thread with its own runtime
    // (gix::Repository is !Send, so we can't use tokio::spawn directly)
    let sync_cache = cache.clone();
    let sync_interval = config.sync.poll_interval_secs;
    std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("failed to build sync runtime");
        rt.block_on(chain_sync::run_sync_loop(sync_cache, sync_interval));
    });

    // Start HTTP server
    let http_cache = cache.clone();
    let http_port = config.server.http_port;
    tokio::spawn(async move {
        let app = http_server::router(http_cache);
        let listener = tokio::net::TcpListener::bind(("0.0.0.0", http_port))
            .await
            .expect("failed to bind HTTP");
        tracing::info!(port = http_port, "HTTP server listening");
        axum::serve(listener, app)
            .await
            .expect("HTTP server error");
    });

    // Start SSH server (runs on main task)
    let ssh = SshServer::new(cache);
    ssh.run(config.server.ssh_port, &config.server.ssh_host_key_path)
        .await?;

    Ok(())
}
