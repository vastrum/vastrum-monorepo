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
