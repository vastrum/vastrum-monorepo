mod common;

use common::*;
use serial_test::serial;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::TempDir;
use vastrum_git_lib::ContractAbiClient;
use vastrum_rpc_client::SentTxBehavior;
use vastrum_shared_types::crypto::ed25519;

#[tokio::test(flavor = "multi_thread")]
#[serial]
async fn test_push_main_branch() {
    let shared = ensure_relay().await;
    let contract =
        ContractAbiClient::new(shared.site_id).with_account_key(ed25519::PrivateKey::from_rng());

    let repo_name = "test_push_main";
    contract.create_repository(repo_name, "").await.await_confirmation().await;

    // Register SSH key.
    let tmp = TempDir::new().unwrap();
    let (priv_key, pub_key) = generate_ssh_keypair(tmp.path());
    let fp = parse_ssh_fingerprint(&pub_key);
    contract.set_ssh_key_fingerprint(repo_name, fp).await.await_confirmation().await;

    // Push main.
    let local = TempDir::new().unwrap();
    init_repo(local.path(), "main", "hello");
    assert_push_ok(git_ssh_push(local.path(), &priv_key, repo_name, "main"));

    // Assert chain state.
    let local_head = git_branch_head(local.path(), "main");
    let state = contract.state().await;
    let repo = state.repo_store.get(&repo_name.to_string()).await.unwrap();

    assert_eq!(repo.branches.len(), 1, "should have exactly one branch");
    assert_eq!(repo.default_branch, "main");
    let on_chain_head = repo.branches.get("main").expect("main branch missing");
    assert_eq!(on_chain_head.0, local_head, "on-chain commit hash mismatch");

    // Commit object must exist in git_object_store
    let commit_obj = state.git_object_store.get(&vastrum_git_lib::Sha1Hash(local_head)).await;
    assert!(commit_obj.is_some(), "commit object missing from git_object_store");
}

#[tokio::test(flavor = "multi_thread")]
#[serial]
async fn test_push_first_branch_becomes_default() {
    let shared = ensure_relay().await;
    let contract =
        ContractAbiClient::new(shared.site_id).with_account_key(ed25519::PrivateKey::from_rng());

    let repo_name = "test_first_wins";
    contract.create_repository(repo_name, "").await.await_confirmation().await;

    let tmp = TempDir::new().unwrap();
    let (priv_key, pub_key) = generate_ssh_keypair(tmp.path());
    let fp = parse_ssh_fingerprint(&pub_key);
    contract.set_ssh_key_fingerprint(repo_name, fp).await.await_confirmation().await;

    let local = TempDir::new().unwrap();
    init_repo(local.path(), "feature/x", "hi");
    assert_push_ok(git_ssh_push(local.path(), &priv_key, repo_name, "feature/x"));

    let local_head = git_branch_head(local.path(), "feature/x");
    let state = contract.state().await;
    let repo = state.repo_store.get(&repo_name.to_string()).await.unwrap();

    assert_eq!(repo.branches.len(), 1);
    assert_eq!(repo.default_branch, "feature/x");
    assert_eq!(repo.branches.get("feature/x").unwrap().0, local_head);
    assert!(state.git_object_store.get(&vastrum_git_lib::Sha1Hash(local_head)).await.is_some());
}

#[tokio::test(flavor = "multi_thread")]
#[serial]
async fn test_push_multi_branch() {
    let shared = ensure_relay().await;
    let contract =
        ContractAbiClient::new(shared.site_id).with_account_key(ed25519::PrivateKey::from_rng());

    let repo_name = "test_multi_branch";
    contract.create_repository(repo_name, "").await.await_confirmation().await;

    let tmp = TempDir::new().unwrap();
    let (priv_key, pub_key) = generate_ssh_keypair(tmp.path());
    let fp = parse_ssh_fingerprint(&pub_key);
    contract.set_ssh_key_fingerprint(repo_name, fp).await.await_confirmation().await;

    // Push main first.
    let local = TempDir::new().unwrap();
    init_repo(local.path(), "main", "base");
    assert_push_ok(git_ssh_push(local.path(), &priv_key, repo_name, "main"));

    // Create feature/y and push it too.
    run_git(local.path(), &["checkout", "-q", "-b", "feature/y"]);
    std::fs::write(local.path().join("feature.txt"), "yo").unwrap();
    run_git(local.path(), &["add", "."]);
    run_git(
        local.path(),
        &["-c", "user.email=t@t", "-c", "user.name=t", "commit", "-q", "-m", "feat"],
    );
    assert_push_ok(git_ssh_push(local.path(), &priv_key, repo_name, "feature/y"));

    let main_head = git_branch_head(local.path(), "main");
    let feature_head = git_branch_head(local.path(), "feature/y");
    assert_ne!(main_head, feature_head, "branches should point to different commits");

    let state = contract.state().await;
    let repo = state.repo_store.get(&repo_name.to_string()).await.unwrap();

    assert_eq!(repo.branches.len(), 2);
    assert_eq!(repo.default_branch, "main");
    assert_eq!(repo.branches.get("main").unwrap().0, main_head);
    assert_eq!(repo.branches.get("feature/y").unwrap().0, feature_head);

    // Both commits must be in the object store
    assert!(state.git_object_store.get(&vastrum_git_lib::Sha1Hash(main_head)).await.is_some());
    assert!(state.git_object_store.get(&vastrum_git_lib::Sha1Hash(feature_head)).await.is_some());
}

#[tokio::test(flavor = "multi_thread")]
#[serial]
async fn test_push_branch_delete() {
    let shared = ensure_relay().await;
    let contract =
        ContractAbiClient::new(shared.site_id).with_account_key(ed25519::PrivateKey::from_rng());

    let repo_name = "test_branch_delete";
    contract.create_repository(repo_name, "").await.await_confirmation().await;

    let tmp = TempDir::new().unwrap();
    let (priv_key, pub_key) = generate_ssh_keypair(tmp.path());
    let fp = parse_ssh_fingerprint(&pub_key);
    contract.set_ssh_key_fingerprint(repo_name, fp).await.await_confirmation().await;

    // Push main and a feature branch.
    let local = TempDir::new().unwrap();
    init_repo(local.path(), "main", "base");
    assert_push_ok(git_ssh_push(local.path(), &priv_key, repo_name, "main"));

    run_git(local.path(), &["checkout", "-q", "-b", "feature/doomed"]);
    std::fs::write(local.path().join("f.txt"), "x").unwrap();
    run_git(local.path(), &["add", "."]);
    run_git(
        local.path(),
        &["-c", "user.email=t@t", "-c", "user.name=t", "commit", "-q", "-m", "x"],
    );
    assert_push_ok(git_ssh_push(local.path(), &priv_key, repo_name, "feature/doomed"));

    // Delete the branch with `:refspec` syntax.
    assert_push_ok(git_ssh_push(local.path(), &priv_key, repo_name, ":feature/doomed"));

    let main_head = git_branch_head(local.path(), "main");
    let repo = contract.state().await.repo_store.get(&repo_name.to_string()).await.unwrap();

    assert_eq!(repo.branches.len(), 1, "should only have main after delete");
    assert!(!repo.branches.contains_key("feature/doomed"));
    assert_eq!(repo.branches.get("main").unwrap().0, main_head);
}

#[tokio::test(flavor = "multi_thread")]
#[serial]
async fn test_push_wrong_ssh_key_rejected() {
    let shared = ensure_relay().await;
    let contract =
        ContractAbiClient::new(shared.site_id).with_account_key(ed25519::PrivateKey::from_rng());

    let repo_name = "test_wrong_key";
    contract.create_repository(repo_name, "").await.await_confirmation().await;

    // Register key A.
    let tmp = TempDir::new().unwrap();
    let (_priv_a, pub_a) = generate_ssh_keypair(tmp.path());
    contract
        .set_ssh_key_fingerprint(repo_name, parse_ssh_fingerprint(&pub_a))
        .await
        .await_confirmation()
        .await;

    // Push with key B — should fail.
    let tmp_b = TempDir::new().unwrap();
    let (priv_b, _pub_b) = generate_ssh_keypair(tmp_b.path());

    let local = TempDir::new().unwrap();
    init_repo(local.path(), "main", "hi");
    let out = git_ssh_push(local.path(), &priv_b, repo_name, "main");
    assert!(!out.status.success(), "push with wrong key should have failed");

    // Verify it failed for the right reason (auth), not some other error.
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("not authorized"),
        "expected auth rejection, got stderr: {}",
        stderr
    );

    // Chain state should still be empty.
    let repo = contract.state().await.repo_store.get(&repo_name.to_string()).await.unwrap();
    assert!(repo.branches.is_empty());
}

#[tokio::test(flavor = "multi_thread")]
#[serial]
async fn test_push_then_clone() {
    let shared = ensure_relay().await;
    let contract =
        ContractAbiClient::new(shared.site_id).with_account_key(ed25519::PrivateKey::from_rng());

    let repo_name = "test_push_then_clone";
    contract.create_repository(repo_name, "").await.await_confirmation().await;

    let tmp = TempDir::new().unwrap();
    let (priv_key, pub_key) = generate_ssh_keypair(tmp.path());
    contract
        .set_ssh_key_fingerprint(repo_name, parse_ssh_fingerprint(&pub_key))
        .await
        .await_confirmation()
        .await;

    // SSH push a repo with specific content.
    let local = TempDir::new().unwrap();
    run_git(local.path(), &["init", "-q", "-b", "main"]);
    std::fs::write(local.path().join("README.md"), "pushed via ssh").unwrap();
    std::fs::create_dir(local.path().join("src")).unwrap();
    std::fs::write(local.path().join("src/lib.rs"), "// ssh push").unwrap();
    run_git(local.path(), &["add", "."]);
    run_git(
        local.path(),
        &["-c", "user.email=t@t", "-c", "user.name=t", "commit", "-q", "-m", "init"],
    );
    assert_push_ok(git_ssh_push(local.path(), &priv_key, repo_name, "main"));

    // HTTPS clone from the relay and verify contents match.
    let cloned = TempDir::new().unwrap();
    let url = format!("http://127.0.0.1:8080/{}", repo_name);
    let out = Command::new("git")
        .args(["clone", &url, cloned.path().to_str().unwrap()])
        .output()
        .unwrap();
    assert!(out.status.success(), "clone failed: {}", String::from_utf8_lossy(&out.stderr));

    assert_eq!(
        std::fs::read_to_string(cloned.path().join("README.md")).unwrap(),
        "pushed via ssh"
    );
    assert_eq!(
        std::fs::read_to_string(cloned.path().join("src/lib.rs")).unwrap(),
        "// ssh push"
    );
}

#[tokio::test(flavor = "multi_thread")]
#[serial]
async fn test_push_incremental() {
    let shared = ensure_relay().await;
    let contract =
        ContractAbiClient::new(shared.site_id).with_account_key(ed25519::PrivateKey::from_rng());

    let repo_name = "test_push_incremental";
    contract.create_repository(repo_name, "").await.await_confirmation().await;

    let tmp = TempDir::new().unwrap();
    let (priv_key, pub_key) = generate_ssh_keypair(tmp.path());
    contract
        .set_ssh_key_fingerprint(repo_name, parse_ssh_fingerprint(&pub_key))
        .await
        .await_confirmation()
        .await;

    // First push.
    let local = TempDir::new().unwrap();
    init_repo(local.path(), "main", "v1");
    let first_head = git_branch_head(local.path(), "main");
    assert_push_ok(git_ssh_push(local.path(), &priv_key, repo_name, "main"));

    // Second commit + push (incremental — collect_all_objects with stop_at=first_head).
    std::fs::write(local.path().join("README.md"), "v2").unwrap();
    run_git(local.path(), &["add", "."]);
    run_git(
        local.path(),
        &["-c", "user.email=t@t", "-c", "user.name=t", "commit", "-q", "-m", "v2"],
    );
    let second_head = git_branch_head(local.path(), "main");
    assert_ne!(first_head, second_head);
    assert_push_ok(git_ssh_push(local.path(), &priv_key, repo_name, "main"));

    // Chain should point to second commit. Both commits should be in the object store.
    let state = contract.state().await;
    let repo = state.repo_store.get(&repo_name.to_string()).await.unwrap();
    assert_eq!(repo.branches.get("main").unwrap().0, second_head);
    assert!(state.git_object_store.get(&vastrum_git_lib::Sha1Hash(first_head)).await.is_some());
    assert!(state.git_object_store.get(&vastrum_git_lib::Sha1Hash(second_head)).await.is_some());
}

#[tokio::test(flavor = "multi_thread")]
#[serial]
async fn test_push_already_up_to_date() {
    let shared = ensure_relay().await;
    let contract =
        ContractAbiClient::new(shared.site_id).with_account_key(ed25519::PrivateKey::from_rng());

    let repo_name = "test_already_up_to_date";
    contract.create_repository(repo_name, "").await.await_confirmation().await;

    let tmp = TempDir::new().unwrap();
    let (priv_key, pub_key) = generate_ssh_keypair(tmp.path());
    contract
        .set_ssh_key_fingerprint(repo_name, parse_ssh_fingerprint(&pub_key))
        .await
        .await_confirmation()
        .await;

    let local = TempDir::new().unwrap();
    init_repo(local.path(), "main", "hi");
    assert_push_ok(git_ssh_push(local.path(), &priv_key, repo_name, "main"));

    // Second push without any new commits — should succeed (no-op).
    let out = git_ssh_push(local.path(), &priv_key, repo_name, "main");
    assert!(
        out.status.success(),
        "idempotent push failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    let head = git_branch_head(local.path(), "main");
    let repo = contract.state().await.repo_store.get(&repo_name.to_string()).await.unwrap();
    assert_eq!(repo.branches.len(), 1);
    assert_eq!(repo.branches.get("main").unwrap().0, head);
}

#[tokio::test(flavor = "multi_thread")]
#[serial]
async fn test_clone_multi_branch() {
    let shared = ensure_relay().await;
    let contract =
        ContractAbiClient::new(shared.site_id).with_account_key(ed25519::PrivateKey::from_rng());

    let repo_name = "test_clone_multi_branch";
    contract.create_repository(repo_name, "").await.await_confirmation().await;

    let tmp = TempDir::new().unwrap();
    let (priv_key, pub_key) = generate_ssh_keypair(tmp.path());
    contract
        .set_ssh_key_fingerprint(repo_name, parse_ssh_fingerprint(&pub_key))
        .await
        .await_confirmation()
        .await;

    // Push main.
    let local = TempDir::new().unwrap();
    init_repo(local.path(), "main", "base");
    assert_push_ok(git_ssh_push(local.path(), &priv_key, repo_name, "main"));

    // Push feature/y.
    run_git(local.path(), &["checkout", "-q", "-b", "feature/y"]);
    std::fs::write(local.path().join("feature.txt"), "feature content").unwrap();
    run_git(local.path(), &["add", "."]);
    run_git(
        local.path(),
        &["-c", "user.email=t@t", "-c", "user.name=t", "commit", "-q", "-m", "f"],
    );
    assert_push_ok(git_ssh_push(local.path(), &priv_key, repo_name, "feature/y"));

    // Clone via HTTPS — should get both branches.
    let cloned = TempDir::new().unwrap();
    let url = format!("http://127.0.0.1:8080/{}", repo_name);
    let out = Command::new("git")
        .args(["clone", &url, cloned.path().to_str().unwrap()])
        .output()
        .unwrap();
    assert!(out.status.success(), "clone failed: {}", String::from_utf8_lossy(&out.stderr));

    // `git branch -r` should list both remote branches.
    let out = Command::new("git")
        .args(["branch", "-r"])
        .current_dir(cloned.path())
        .output()
        .unwrap();
    let branches = String::from_utf8_lossy(&out.stdout);
    assert!(branches.contains("origin/main"), "main missing: {}", branches);
    assert!(branches.contains("origin/feature/y"), "feature/y missing: {}", branches);
}

/// Generate an SSH keypair, return (private_key_file, ssh_pubkey_string).
fn generate_ssh_keypair(tmp: &Path) -> (PathBuf, String) {
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

fn run_git(dir: &Path, args: &[&str]) {
    let out = Command::new("git").args(args).current_dir(dir).output().unwrap();
    assert!(
        out.status.success(),
        "git {:?} failed: {}",
        args,
        String::from_utf8_lossy(&out.stderr)
    );
}

/// Read the commit SHA of a local branch as raw 20 bytes.
fn git_branch_head(dir: &Path, branch: &str) -> [u8; 20] {
    let repo = gix::open(dir).unwrap();
    let reference = repo.find_reference(&format!("refs/heads/{}", branch)).unwrap();
    let oid = match reference.target() {
        gix::refs::TargetRef::Object(oid) => oid.to_owned(),
        gix::refs::TargetRef::Symbolic(_) => panic!("expected direct ref"),
    };
    oid.as_bytes().try_into().unwrap()
}

/// Initialize a local git repo with one commit on the given branch.
fn init_repo(dir: &Path, branch: &str, content: &str) {
    run_git(dir, &["init", "-q", "-b", branch]);
    std::fs::write(dir.join("README.md"), content).unwrap();
    run_git(dir, &["add", "."]);
    run_git(
        dir,
        &["-c", "user.email=test@test", "-c", "user.name=test", "commit", "-q", "-m", "init"],
    );
}

fn git_ssh_push(
    local: &Path,
    priv_key: &Path,
    repo_name: &str,
    refspec: &str,
) -> std::process::Output {
    let ssh_cmd = format!(
        "ssh -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null -o IdentitiesOnly=yes -i {} -p 2222",
        priv_key.display()
    );
    let url = format!("ssh://git@127.0.0.1/{}", repo_name);
    Command::new("git")
        .args(["push", &url, refspec])
        .env("GIT_SSH_COMMAND", &ssh_cmd)
        .current_dir(local)
        .output()
        .unwrap()
}

fn assert_push_ok(out: std::process::Output) {
    assert!(
        out.status.success(),
        "git push failed:\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
}
