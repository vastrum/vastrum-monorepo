#![allow(dead_code, unused_imports)]
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
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

/// Generate an SSH keypair, return (private_key_file, ssh_pubkey_string).
pub fn generate_ssh_keypair(tmp: &Path) -> (PathBuf, String) {
    use ssh_key::{Algorithm, LineEnding, PrivateKey};
    let key = PrivateKey::random(&mut rand::thread_rng(), Algorithm::Ed25519).unwrap();
    let priv_path = tmp.join("id_ed25519");
    key.write_openssh_file(&priv_path, LineEnding::LF).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&priv_path, std::fs::Permissions::from_mode(0o600)).unwrap();
    }
    let pub_key = key.public_key().to_openssh().unwrap();
    (priv_path, pub_key)
}

pub fn run_git(dir: &Path, args: &[&str]) {
    let out = Command::new("git").args(args).current_dir(dir).output().unwrap();
    assert!(
        out.status.success(),
        "git {:?} failed: {}",
        args,
        String::from_utf8_lossy(&out.stderr)
    );
}

/// Read the commit SHA of a local branch as raw 20 bytes.
pub fn git_branch_head(dir: &Path, branch: &str) -> [u8; 20] {
    let repo = gix::open(dir).unwrap();
    let reference = repo.find_reference(&format!("refs/heads/{}", branch)).unwrap();
    let oid = match reference.target() {
        gix::refs::TargetRef::Object(oid) => oid.to_owned(),
        gix::refs::TargetRef::Symbolic(_) => panic!("expected direct ref"),
    };
    oid.as_bytes().try_into().unwrap()
}

/// Initialize a local git repo with one commit on the given branch.
pub fn init_repo(dir: &Path, branch: &str, content: &str) {
    run_git(dir, &["init", "-q", "-b", branch]);
    std::fs::write(dir.join("README.md"), content).unwrap();
    run_git(dir, &["add", "."]);
    run_git(
        dir,
        &["-c", "user.email=test@test", "-c", "user.name=test", "commit", "-q", "-m", "init"],
    );
}

fn git_ssh_env(priv_key: &Path) -> String {
    format!(
        "ssh -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null -o IdentitiesOnly=yes -i {} -p 2222",
        priv_key.display()
    )
}

pub fn git_ssh_push(local: &Path, priv_key: &Path, repo_name: &str, refspec: &str) -> Output {
    let url = format!("ssh://git@127.0.0.1/{}", repo_name);
    Command::new("git")
        .args(["push", &url, refspec])
        .env("GIT_SSH_COMMAND", git_ssh_env(priv_key))
        .current_dir(local)
        .output()
        .unwrap()
}

pub fn git_ssh_clone(target: &Path, priv_key: &Path, repo_name: &str) -> Output {
    let url = format!("ssh://git@127.0.0.1/{}", repo_name);
    Command::new("git")
        .args(["clone", &url, target.to_str().unwrap()])
        .env("GIT_SSH_COMMAND", git_ssh_env(priv_key))
        .output()
        .unwrap()
}

pub fn assert_push_ok(out: Output) {
    assert!(
        out.status.success(),
        "git push failed:\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
}
