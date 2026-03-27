//debug_assertions is false if built with cargo --release
fn build_frontend() {
    run("npm install", "../frontend");
    let script = if cfg!(debug_assertions) { "build" } else { "build:prod" };
    run(&format!("npm run {script}"), "../frontend");
}

#[tokio::main]
async fn main() {
    build_contract("../contract", "../contract/out");
    build_frontend();
    let html = std::fs::read_to_string("../frontend/dist/index.html").unwrap();
    let brotli_html_content = vastrum_shared_types::compression::brotli::brotli_compress_html(&html);
    let client =
        ContractAbiClient::deploy("../contract/out/contract.wasm", brotli_html_content).await;
    let site_id = client.site_id();
    register_domain(site_id, GITTER_DOMAIN).await.await_confirmation().await;
    register_domain(site_id, "index").await.await_confirmation().await;

    deploy_example_repos(site_id).await;

    let monorepo_key = ed25519::PrivateKey::from_rng();
    deploy_monorepo(site_id, &monorepo_key).await;

    println!();
    println!("=== Deploy complete ===");
    println!("monorepo_key: {monorepo_key}");
}

async fn deploy_monorepo(site_id: Sha256Digest, monorepo_key: &ed25519::PrivateKey) {
    let client = ContractAbiClient::new(site_id).with_account_key(monorepo_key.clone());

    client
        .create_repository("vastrum", "Vastrum is a protocol for hosting decentralized websites.")
        .await
        .await_confirmation()
        .await;

    let repo = create_monorepo_snapshot();
    push_to_repo(repo.path_str(), "vastrum", &client, None).await.unwrap();
}

fn create_monorepo_snapshot() -> TestRepo {
    // Find monorepo root
    let monorepo_root = std::path::Path::new("../../..").canonicalize().unwrap();

    // Get all tracked files via git ls-files
    let output = std::process::Command::new("git")
        .args(["ls-files"])
        .current_dir(&monorepo_root)
        .output()
        .unwrap();
    assert!(output.status.success(), "git ls-files failed");

    let file_list = String::from_utf8(output.stdout).unwrap();
    let mut builder = TestRepoBuilder::new();

    for relative_path in file_list.lines() {
        if relative_path.is_empty() {
            continue;
        }
        let full_path = monorepo_root.join(relative_path);
        match std::fs::read(&full_path) {
            Ok(contents) => {
                builder = builder.file(relative_path, &contents);
            }
            Err(e) => {
                eprintln!("Warning: skipping {relative_path}: {e}");
            }
        }
    }

    return builder.build();
}

async fn deploy_example_repos(site_id: Sha256Digest) {
    let client = ContractAbiClient::new(site_id).with_account_key(ed25519::PrivateKey::from_rng());

    // Create base example repo
    client.create_repository("example-repo", "Example repo").await.await_confirmation().await;

    // Build base repo and push
    let repo = TestRepoBuilder::new()
        .file("README.md", b"# Example Repo\nAn example repository.")
        .file("src/main.rs", b"fn main() {\n    println!(\"Hello, world!\");\n}\n")
        .file(
            "Cargo.toml",
            b"[package]\nname = \"example\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
        )
        .file(".gitignore", b"/target\n")
        .build();
    push_to_repo(repo.path_str(), "example-repo", &client, None).await.unwrap();

    // Create issue with replies
    client
        .create_issue(
            "This is an example issue",
            "Issues could be used to coordinate problems, currently this data is also used for the discussion tab.",
            "example-repo",
        )
        .await
        .await_confirmation()
        .await;

    client
        .reply_to_issue("This is an example reply to the issue", "example-repo", 0)
        .await
        .await_confirmation()
        .await;

    client
        .reply_to_issue("This is another reply", "example-repo", 0)
        .await
        .await_confirmation()
        .await;

    // Fork and add a commit on top (shared ancestry with base)
    client.fork_repository("example-repo-fork", "example-repo").await.await_confirmation().await;

    repo.add_commit(&[
        ("README.md", b"# Example Repo\nAn example repository.\n\nForked with changes."),
        ("src/main.rs", b"fn main() {\n    println!(\"Hello from fork!\");\n    greet();\n}\n\nfn greet() {\n    println!(\"Greetings!\");\n}\n"),
        ("Cargo.toml", b"[package]\nname = \"example\"\nversion = \"0.1.0\"\nedition = \"2024\"\n"),
        (".gitignore", b"/target\n"),
    ]);
    push_to_repo(repo.path_str(), "example-repo-fork", &client, None).await.unwrap();

    // Create PR with replies
    client
        .create_pull_request(
            "example-repo",
            "example-repo-fork",
            "Pull request example",
            "This pull request adds a greeting function.",
        )
        .await
        .await_confirmation()
        .await;

    client
        .reply_to_pull_request("Good pull request.", "example-repo", 0)
        .await
        .await_confirmation()
        .await;

    client
        .reply_to_pull_request("Bad pull request.", "example-repo", 0)
        .await
        .await_confirmation()
        .await;
}

use vastrum_native_lib::deployers::{
    build::{build_contract, run},
    deploy::register_domain,
};
use vastrum_rpc_client::SentTxBehavior;
use vastrum_shared_types::crypto::{ed25519, sha256::Sha256Digest};
use vastrum_git_lib::{
    ContractAbiClient,
    config::GITTER_DOMAIN,
    native::upload::push_to_repo,
    testing::test_helpers::{TestRepo, TestRepoBuilder},
};
