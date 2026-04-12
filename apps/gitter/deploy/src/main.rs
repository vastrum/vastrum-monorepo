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
    let brotli_html_content =
        vastrum_shared_types::compression::brotli::brotli_compress_html(&html);
    let relay_key = load_or_generate_relay_key();
    let client = ContractAbiClient::deploy(
        "../contract/out/contract.wasm",
        brotli_html_content,
        relay_key.public_key(),
    )
    .await;
    let site_id = client.site_id();
    register_domain(site_id, GITTER_DOMAIN).await.await_confirmation().await;
    register_domain(site_id, "index").await.await_confirmation().await;
    register_domain(site_id, site_id.to_string()).await.await_confirmation().await;
    // static testnet site_id registration in case of network redeployment
    // causing site to have different site_id and causing dead links
    let static_site_id =
        Sha256Digest::from_string("yts27rvo7ppzq5rrjyavmfwecrbyc5ksldmitiggycetgh6zguoa").unwrap();
    register_domain(site_id, static_site_id.to_string()).await.await_confirmation().await;

    deploy_example_repos(site_id).await;

    let monorepo_key = ed25519::PrivateKey::from_rng();
    deploy_monorepo(site_id, &monorepo_key).await;

    println!();
    println!("=== Deploy complete ===");
    println!("monorepo_key: {monorepo_key}");
    println!("relay_key: {relay_key}");
}

async fn deploy_monorepo(site_id: Sha256Digest, monorepo_key: &ed25519::PrivateKey) {
    let client = ContractAbiClient::new(site_id).with_account_key(monorepo_key.clone());

    client
        .create_repository("vastrum", "Vastrum is a protocol for hosting decentralized websites.")
        .await
        .await_confirmation()
        .await;

    let monorepo_root = std::path::Path::new("../../..").canonicalize().unwrap();
    let monorepo_path = monorepo_root.to_str().unwrap();
    push_to_repo(monorepo_path, "vastrum", &client, None).await.unwrap();

    // Create example issue with replies
    client
        .create_issue(
            "Chatter DOS issue",
            "
Currently any user can write to any key, this means you could DOS other users by overwriting their inboxes with garbage data.

Instead could treat the shared seed as a private key, then write to the public key of that private key.

The contract would then require all writes to be to an ed25519 public key and require the user to provide a signature for the content.

Letterer had similar problem, this was solved by each document key being a public key.

```rust
#[authenticated]
pub fn save_document(
    &mut self,
    document_key: Ed25519PublicKey,
    signature: Ed25519Signature,
    operation: DocumentWriteOperation,
) {
    let encoded = borsh::to_vec(&operation).unwrap();
    let hash = runtime::sha256(&encoded);
    let signature_matches_document_key = document_key.verify(&hash, &signature);
    if !signature_matches_document_key {
        return;
    }
    self.documents.set(&document_key, operation.content);
    self.doc_metadata.set(&document_key, operation.metadata);
}
```

https://gitter.vastrum.net/repo/vastrum/tree/apps/letterer/contract/src/lib.rs

and frontend WASM

```rust
pub async fn save_document(
    doc_priv: &PrivateKey,
    content: &DocumentContent,
    meta: &DocumentMeta,
) -> String {
    let encrypted_content = encrypt_content(doc_priv, content);
    let encrypted_meta = encrypt_metadata(doc_priv, meta);
    let operation = DocumentWriteOperation { content: encrypted_content, metadata: encrypted_meta };
    let signature = sign_document(doc_priv, &operation);
    let doc_pub = doc_priv.public_key();
    let client = new_client();
    let sent_tx = client.save_document(doc_pub, signature, operation).await;
    return sent_tx.tx_hash().to_string();
}
```

https://gitter.vastrum.net/repo/vastrum/tree/apps/letterer/frontend/wasm/src/encryption.rs

current chatter

```rust
pub fn write_to_inbox(&mut self, inbox_id: String, content: String) {
    self.inbox.set(&inbox_id, content);
}
```

https://gitter.vastrum.net/repo/vastrum/tree/apps/chatter/contract/src/lib.rs


",
            "vastrum",
        )
        .await
        .await_confirmation()
        .await;

    client.reply_to_issue("Issue reply test", "vastrum", 0).await.await_confirmation().await;

    client.fork_repository("vastrum-pr-fork", "vastrum").await.await_confirmation().await;

    let new_readme: &[u8] = b"# Vastrum

Experimental protocol for decentralized website hosting. [Docs](https://xpkeuoccopibhnakya3luhrsphalhnqo2ifmxe65murdjft54n3q.vastrum.net).

## Setup

1. Install [Rust](https://rustup.rs)
2. Install [Node.js](https://nodejs.org)
3. Run:

```bash
sudo apt install clang build-essential liblz4-dev
rustup target add wasm32-unknown-unknown
cargo install wasm-pack
cargo install mdbook
```

## Run website locally

```bash
cd apps/gitter && cargo run -p vastrum-cli -- run-dev
```

## Deploy all websites locally

```bash
make deploy-all-localnet
```

## Scaffold project

### Install vastrum-cli

**Prebuilt binary:**

```bash
curl -sSf https://raw.githubusercontent.com/vastrum/vastrum-monorepo/HEAD/tooling/cli/install.sh | sh
```

**From source:**

```
make cli_install
```

### Scaffold options

```
vastrum-cli init <name> --template site
vastrum-cli init <name> --template eth_dapp
```

### Project structure

6K lines Rust for vastrum-node, 30K lines Rust for whole monorepo (excluding Helios + jmt-main).

- **vastrum-node** - Blockchain node
- **apps** - Prototype apps
- **runtime** - Libs, tooling, tests for smart contract runtime
- **shared-types** - Shared internal lib
- **web-client** - Frontend served by vastrum.net
  - **app** - Frontend
  - **helios-worker** - Web worker hosting helios
  - **integration tests**
- **webrtc-direct** - WebRTC-direct impl
- **tooling** - CLI, app libs
- **vendored-helios** - https://github.com/a16z/helios
- **vendored-jmt-main** - https://github.com/penumbra-zone/jmt";

    let fork_dir = build_fork_commit(monorepo_path, &monorepo_root, new_readme);
    push_to_repo(fork_dir.path().to_str().unwrap(), "vastrum-pr-fork", &client, None)
        .await
        .unwrap();

    client
        .create_pull_request(
            "vastrum",
            "master",
            "vastrum-pr-fork",
            "master",
            "Improve README",
            "Improved readme",
        )
        .await
        .await_confirmation()
        .await;

    client.reply_to_pull_request("PR reply example", "vastrum", 0).await.await_confirmation().await;
}

async fn deploy_example_repos(site_id: Sha256Digest) {
    let client = ContractAbiClient::new(site_id).with_account_key(ed25519::PrivateKey::from_rng());

    // Create base example repo
    client.create_repository("example-repo", "Example repo").await.await_confirmation().await;

    // Build base repo and push
    let now_secs =
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs()
            as i64;
    let repo = TestRepoBuilder::new()
        .time(now_secs)
        .file("README.md", b"# Example Repo")
        .file("src/main.rs", b"fn main() {\n    println!(\"Hello world\");\n}\n")
        .file(
            "Cargo.toml",
            b"[package]\nname = \"example\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
        )
        .file(".gitignore", b"/target\n")
        .build();
    push_to_repo(repo.path_str(), "example-repo", &client, None).await.unwrap();

    // Create issue with replies
    client
        .create_issue("Project needs better name", "The name is bad", "example-repo")
        .await
        .await_confirmation()
        .await;

    client.reply_to_issue("I disagree", "example-repo", 0).await.await_confirmation().await;

    client.reply_to_issue("I agree", "example-repo", 0).await.await_confirmation().await;

    // Fork and add a commit on top (shared ancestry with base)
    client.fork_repository("example-repo-fork", "example-repo").await.await_confirmation().await;

    repo.add_commit(&[
        ("README.md", b"# Improved Example Repo"),
        ("src/main.rs", b"fn main() {\n    println!(\"Goodbye world\");\n}\n"),
        ("Cargo.toml", b"[package]\nname = \"example\"\nversion = \"0.1.0\"\nedition = \"2024\"\n"),
        (".gitignore", b"/target\n"),
    ]);
    push_to_repo(repo.path_str(), "example-repo-fork", &client, None).await.unwrap();

    // Create PR with replies
    client
        .create_pull_request(
            "example-repo",
            "master",
            "example-repo-fork",
            "master",
            "pull requester",
            "Goodbye world",
        )
        .await
        .await_confirmation()
        .await;

    client
        .reply_to_pull_request("Good pull request", "example-repo", 0)
        .await
        .await_confirmation()
        .await;

    client
        .reply_to_pull_request("Bad pull request", "example-repo", 0)
        .await
        .await_confirmation()
        .await;
}

fn build_fork_commit(
    monorepo_path: &str,
    monorepo_root: &std::path::Path,
    new_readme: &[u8],
) -> tempfile::TempDir {
    let fork_dir = tempfile::TempDir::new().unwrap();
    let _ = gix::init(fork_dir.path()).unwrap();

    let source_repo = gix::open(monorepo_path).unwrap();
    let source_objects = source_repo.common_dir().join("objects");
    let objects_abs = if source_objects.is_absolute() {
        source_objects
    } else {
        monorepo_root.join(".git").join("objects")
    };
    let alternates_path = fork_dir.path().join(".git/objects/info/alternates");
    std::fs::create_dir_all(alternates_path.parent().unwrap()).unwrap();
    std::fs::write(&alternates_path, format!("{}\n", objects_abs.display())).unwrap();

    let fork_repo = gix::open(fork_dir.path()).unwrap();

    let source_head = source_repo.head_id().unwrap().detach();
    let source_commit = source_repo.find_commit(source_head).unwrap();
    let source_tree_id = source_commit.tree_id().unwrap().detach();
    let source_tree = source_repo.find_tree(source_tree_id).unwrap();

    let new_blob_id =
        fork_repo.write_object(&gix::objs::Blob { data: new_readme.to_vec() }).unwrap().detach();

    let mut new_entries: Vec<gix::objs::tree::Entry> = Vec::new();
    for entry_result in source_tree.iter() {
        let entry = entry_result.unwrap();
        let filename = entry.filename();
        let oid = if filename.to_string() == "README.md" { new_blob_id } else { entry.object_id() };
        new_entries.push(gix::objs::tree::Entry {
            mode: entry.mode(),
            filename: filename.to_owned(),
            oid,
        });
    }
    let new_tree_id =
        fork_repo.write_object(&gix::objs::Tree { entries: new_entries }).unwrap().detach();

    let now_secs =
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs()
            as i64;
    let sig = gix::actor::Signature {
        name: "vastrum-deploy".into(),
        email: "deploy@vastrum.local".into(),
        time: gix::date::Time::new(now_secs, 0),
    };
    let new_commit_id = fork_repo
        .write_object(&gix::objs::Commit {
            tree: new_tree_id,
            parents: vec![source_head].into(),
            author: sig.clone(),
            committer: sig,
            encoding: None,
            message: "Improve README".into(),
            extra_headers: vec![],
        })
        .unwrap()
        .detach();

    let branch_ref: gix::refs::FullName = "refs/heads/master".try_into().unwrap();
    fork_repo
        .edit_reference(gix::refs::transaction::RefEdit {
            change: gix::refs::transaction::Change::Update {
                log: gix::refs::transaction::LogChange {
                    mode: gix::refs::transaction::RefLog::AndReference,
                    force_create_reflog: false,
                    message: "update".into(),
                },
                expected: gix::refs::transaction::PreviousValue::Any,
                new: gix::refs::Target::Object(new_commit_id),
            },
            name: branch_ref.clone(),
            deref: false,
        })
        .unwrap();
    fork_repo
        .edit_reference(gix::refs::transaction::RefEdit {
            change: gix::refs::transaction::Change::Update {
                log: gix::refs::transaction::LogChange {
                    mode: gix::refs::transaction::RefLog::AndReference,
                    force_create_reflog: false,
                    message: "set HEAD".into(),
                },
                expected: gix::refs::transaction::PreviousValue::Any,
                new: gix::refs::Target::Symbolic(branch_ref),
            },
            name: "HEAD".try_into().unwrap(),
            deref: false,
        })
        .unwrap();

    fork_dir
}

fn load_or_generate_relay_key() -> ed25519::PrivateKey {
    for path in ["../../genesis/git-relay/relay.key", "../relay.key"] {
        if let Ok(s) = std::fs::read_to_string(path) {
            if let Some(key) = ed25519::PrivateKey::try_from_string(s.trim().to_string()) {
                println!("loaded relay key from {path}");
                return key;
            }
        }
    }
    let key = ed25519::PrivateKey::from_rng();
    std::fs::write("../relay.key", key.to_string()).unwrap();
    println!("generated new relay key at ../relay.key");
    key
}

use vastrum_git_lib::{
    ContractAbiClient, config::GITTER_DOMAIN, native::upload::push_to_repo,
    testing::test_helpers::TestRepoBuilder,
};
use vastrum_native_lib::deployers::{
    build::{build_contract, run},
    deploy::register_domain,
};
use vastrum_rpc_client::SentTxBehavior;
use vastrum_shared_types::crypto::{ed25519, sha256::Sha256Digest};
