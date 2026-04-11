mod common;

use common::*;
use serial_test::serial;
use std::process::Command;
use tempfile::TempDir;
use vastrum_git_lib::ContractAbiClient;
use vastrum_git_lib::native::upload::push_to_repo;
use vastrum_git_lib::testing::test_helpers::TestRepoBuilder;
use vastrum_rpc_client::SentTxBehavior;
use vastrum_shared_types::crypto::ed25519;

#[tokio::test(flavor = "multi_thread")]
#[serial]
async fn test_http_clone_basic() {
    let shared = ensure_relay().await;
    let contract =
        ContractAbiClient::new(shared.site_id).with_account_key(ed25519::PrivateKey::from_rng());

    let repo_name = "test_http_clone_basic";
    contract.create_repository(repo_name, "").await.await_confirmation().await;

    // Populate the repo via the CLI push path (uses main branch).
    let repo = TestRepoBuilder::new()
        .file("README.md", b"# hello")
        .file("src/main.rs", b"fn main() {}")
        .build();
    push_to_repo(repo.path_str(), repo_name, &contract, None).await.unwrap();

    // Clone via HTTP using the real git binary.
    let target = TempDir::new().unwrap();
    let url = format!("http://127.0.0.1:8080/{}", repo_name);
    let out = Command::new("git")
        .args(["clone", &url, target.path().to_str().unwrap()])
        .output()
        .expect("git clone failed to execute");
    assert!(
        out.status.success(),
        "git clone failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    assert!(target.path().join("README.md").exists());
    assert!(target.path().join("src/main.rs").exists());
    assert_eq!(
        std::fs::read_to_string(target.path().join("README.md")).unwrap(),
        "# hello"
    );
}

/// Clone over SSH using an unregistered key. Upload-pack must be anonymous.
#[tokio::test(flavor = "multi_thread")]
#[serial]
async fn test_ssh_clone_anonymous() {
    let shared = ensure_relay().await;
    let contract =
        ContractAbiClient::new(shared.site_id).with_account_key(ed25519::PrivateKey::from_rng());

    let repo_name = "test_ssh_clone_anonymous";
    contract.create_repository(repo_name, "").await.await_confirmation().await;

    // Populate via the CLI push path — does not need an SSH key.
    let repo = TestRepoBuilder::new()
        .file("README.md", b"anon clone")
        .file("src/main.rs", b"fn main() {}")
        .build();
    push_to_repo(repo.path_str(), repo_name, &contract, None).await.unwrap();

    // Fresh keypair, intentionally NOT registered on the repo.
    let tmp = TempDir::new().unwrap();
    let (priv_key, _pub_key) = generate_ssh_keypair(tmp.path());

    let target = TempDir::new().unwrap();
    let out = git_ssh_clone(&target.path().join("clone"), &priv_key, repo_name);
    assert!(
        out.status.success(),
        "ssh clone failed:\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );

    let cloned = target.path().join("clone");
    assert!(cloned.join("README.md").exists());
    assert!(cloned.join("src/main.rs").exists());
    assert_eq!(
        std::fs::read_to_string(cloned.join("README.md")).unwrap(),
        "anon clone"
    );
}

/// SSH clone of a nonexistent repo must fail (not hang, not silently 0-exit).
#[tokio::test(flavor = "multi_thread")]
#[serial]
async fn test_ssh_clone_nonexistent_repo() {
    let _ = ensure_relay().await;

    let tmp = TempDir::new().unwrap();
    let (priv_key, _) = generate_ssh_keypair(tmp.path());

    let target = TempDir::new().unwrap();
    let out = git_ssh_clone(
        &target.path().join("x"),
        &priv_key,
        "this_repo_does_not_exist",
    );
    assert!(!out.status.success(), "clone of nonexistent repo should fail");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("not found"),
        "expected 'not found' in stderr, got: {}",
        stderr
    );
}

/// End-to-end round-trip for the CloneModal SSH-first UX: push via SSH with a
/// registered key, then clone back via SSH with a different, UNREGISTERED key.
/// Symmetric to test_push_then_clone but both legs use SSH.
#[tokio::test(flavor = "multi_thread")]
#[serial]
async fn test_ssh_clone_push_then_ssh_clone() {
    let shared = ensure_relay().await;
    let contract =
        ContractAbiClient::new(shared.site_id).with_account_key(ed25519::PrivateKey::from_rng());

    let repo_name = "test_ssh_clone_push_then_ssh_clone";
    contract.create_repository(repo_name, "").await.await_confirmation().await;

    // Owner key — registered, used for push.
    let owner_tmp = TempDir::new().unwrap();
    let (owner_priv, owner_pub) = generate_ssh_keypair(owner_tmp.path());
    contract
        .set_ssh_key_fingerprint(repo_name, parse_ssh_fingerprint(&owner_pub))
        .await
        .await_confirmation()
        .await;

    // SSH push content.
    let local = TempDir::new().unwrap();
    run_git(local.path(), &["init", "-q", "-b", "main"]);
    std::fs::write(local.path().join("README.md"), "round trip via ssh").unwrap();
    std::fs::create_dir(local.path().join("src")).unwrap();
    std::fs::write(local.path().join("src/lib.rs"), "// round trip").unwrap();
    run_git(local.path(), &["add", "."]);
    run_git(
        local.path(),
        &["-c", "user.email=t@t", "-c", "user.name=t", "commit", "-q", "-m", "init"],
    );
    assert_push_ok(git_ssh_push(local.path(), &owner_priv, repo_name, "main"));

    // Visitor key — fresh, NOT registered.
    let visitor_tmp = TempDir::new().unwrap();
    let (visitor_priv, _) = generate_ssh_keypair(visitor_tmp.path());

    let target = TempDir::new().unwrap();
    let out = git_ssh_clone(&target.path().join("clone"), &visitor_priv, repo_name);
    assert!(
        out.status.success(),
        "visitor ssh clone failed:\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );

    let cloned = target.path().join("clone");
    assert_eq!(
        std::fs::read_to_string(cloned.join("README.md")).unwrap(),
        "round trip via ssh"
    );
    assert_eq!(
        std::fs::read_to_string(cloned.join("src/lib.rs")).unwrap(),
        "// round trip"
    );
}
