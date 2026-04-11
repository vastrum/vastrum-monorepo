#![allow(dead_code, unused_imports)]
use std::path::PathBuf;
use std::time::Duration;
use tempfile::TempDir;
use tokio::sync::OnceCell;
use vastrum_git_lib::ContractAbiClient;
use vastrum_git_lib::config::GITTER_DOMAIN;
use vastrum_native_lib::deployers::deploy::register_domain;
use vastrum_shared_types::crypto::{ed25519, sha256::Sha256Digest};

pub struct SharedRelay {
    pub site_id: Sha256Digest,
    // Held to keep the relay key file alive for the lifetime of the test suite.
    _relay_data: TempDir,
}

static SHARED: OnceCell<SharedRelay> = OnceCell::const_new();

pub async fn ensure_relay() -> &'static SharedRelay {
    SHARED
        .get_or_init(|| async {
            // 1. Localnet — starts it once (OnceLock inside) and sets VASTRUM_LOCALNET.
            vastrum_native_lib::test_support::ensure_localnet("../contract", "../contract/out");

            // 2. Deploy contract with a fresh relay key.
            let relay_key = ed25519::PrivateKey::from_rng();
            let client = ContractAbiClient::deploy(
                "../contract/out/contract.wasm",
                vec![],
                relay_key.public_key(),
            )
            .await;
            let site_id = client.site_id();

            // 3. Register the domain so the relay's `resolve_domain` call succeeds.
            register_domain(site_id, GITTER_DOMAIN).await.await_confirmation().await;

            // 4. Write relay key to a temp file.
            let relay_data = TempDir::new().unwrap();
            let relay_key_path: PathBuf = relay_data.path().join("relay.key");
            std::fs::write(&relay_key_path, relay_key.to_string()).unwrap();

            // 5. Spawn the relay on a dedicated thread with its own runtime so it
            // survives across tests (each #[tokio::test] creates a fresh runtime).
            std::thread::spawn(move || {
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(async move {
                    if let Err(e) = vastrum_git_relay::run(relay_key_path).await {
                        eprintln!("relay exited: {:?}", e);
                    }
                });
            });

            // 6. Wait for HTTP and SSH ports.
            wait_for_port(8080).await;
            wait_for_port(2222).await;

            SharedRelay { site_id, _relay_data: relay_data }
        })
        .await
}

async fn wait_for_port(port: u16) {
    let addr = format!("127.0.0.1:{}", port);
    for _ in 0..500 {
        if tokio::net::TcpStream::connect(&addr).await.is_ok() {
            return;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    panic!("port {} did not become available", port);
}

/// Compute the SHA256 fingerprint of the base64 data portion of an OpenSSH public key string.
pub fn parse_ssh_fingerprint(ssh_pub_key: &str) -> vastrum_git_lib::SshKeyFingerprint {
    use base64::Engine;
    let parts: Vec<&str> = ssh_pub_key.trim().split_whitespace().collect();
    let bytes = base64::engine::general_purpose::STANDARD.decode(parts[1]).unwrap();
    let hash = vastrum_shared_types::crypto::sha256::sha256_hash(&bytes);
    vastrum_git_lib::SshKeyFingerprint(hash.to_bytes())
}
