const DEFAULT_HTTP_PORT: u16 = 8080;
const DEFAULT_SSH_PORT: u16 = 2222;

pub async fn run(relay_key_path: PathBuf) -> Result<()> {
    let ssh_host_key_path = PathBuf::from("./relay-data/ssh_host_ed25519_key");

    // Read relay private key from file
    let relay_private_key_str = std::fs::read_to_string(&relay_key_path)
        .with_context(|| format!("failed to read relay key from '{}'", relay_key_path.display()))?;
    let relay_private_key =
        ed25519::PrivateKey::try_from_string(relay_private_key_str.trim().to_string())
            .with_context(|| "invalid relay private key")?;

    // for localnet testing check readme
    let is_localnet = std::env::var("VASTRUM_LOCALNET").is_ok();
    if is_localnet {
        tracing::info!("localnet mode: using existing RPC on port {}", HTTP_RPC_PORT);
    } else {
        tracing::info!("starting embedded full node...");
        let tmp = tempfile::tempdir()?;
        let keystore_path = tmp.path().join("keystore.bin");
        let mut node = tokio::spawn(async move {
            let _tmp = tmp;
            vastrum_node::start_node_production(keystore_path, true).await
        });
        tokio::select! {
            _ = wait_for_rpc_server() => {
                tracing::info!("embedded node RPC ready on port {}", HTTP_RPC_PORT);
            }
            result = &mut node => {
                match result {
                    Ok(()) => anyhow::bail!("node exited unexpectedly"),
                    Err(e) => anyhow::bail!("node panicked: {}", e),
                }
            }
        }
    }

    // Point all RPC clients at the local embedded node
    unsafe {
        std::env::set_var(
            "RPC_URL_HACK_FOR_GIT_RELAY",
            format!("http://127.0.0.1:{}", HTTP_RPC_PORT),
        )
    };

    // Wait for gitter contract to be deployed (retries until available)
    let http = NativeHttpClient::new();
    let site_id = loop {
        match http.resolve_domain(GITTER_DOMAIN.to_string()).await {
            Ok(Some(id)) => break id,
            _ => {
                tracing::info!("waiting for gitter contract deployment...");
                tokio::time::sleep(Duration::from_secs(5)).await;
            }
        }
    };

    tracing::info!(site_id = %site_id, "resolved gitter contract");

    // Create contract client authenticated with relay key
    let site_key = derive_site_key(&relay_private_key, site_id);
    let contract = Arc::new(ContractAbiClient::new(site_id).with_account_key(site_key));

    // Start HTTP server
    let http_contract = contract.clone();
    tokio::spawn(async move {
        let app = http_server::router(http_contract);
        let listener = tokio::net::TcpListener::bind(("0.0.0.0", DEFAULT_HTTP_PORT))
            .await
            .expect("failed to bind HTTP");
        tracing::info!(port = DEFAULT_HTTP_PORT, "HTTP server listening");
        axum::serve(listener, app).await.expect("HTTP server error");
    });

    // Start SSH server (blocks forever)
    let ssh = SshServer::new(contract);
    ssh.run(DEFAULT_SSH_PORT, &ssh_host_key_path).await?;

    Ok(())
}

async fn wait_for_rpc_server() {
    let addr = SocketAddr::from(([127, 0, 0, 1], HTTP_RPC_PORT));
    while tokio::net::TcpStream::connect(addr).await.is_err() {
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
}

mod http_server;
mod materialize;
mod push_handler;
mod ssh_server;

use anyhow::{Context, Result};
use ssh_server::SshServer;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use vastrum_git_lib::ContractAbiClient;
use vastrum_git_lib::config::GITTER_DOMAIN;
use vastrum_native_lib::NativeHttpClient;
use vastrum_shared_types::crypto::{ed25519, site_key::derive_site_key};
use vastrum_shared_types::ports::HTTP_RPC_PORT;
