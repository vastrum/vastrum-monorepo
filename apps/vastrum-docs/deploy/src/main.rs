use vastrum_docs_abi::*;

use vastrum_native_lib::deployers::{
    build::{build_contract, run},
    deploy::register_domain,
};
use vastrum_shared_types;

fn build_docs_spa() {
    run("mdbook build", "..");
    run("python3 build_spa.py", "..");
}

#[tokio::main]
async fn main() {
    build_contract("../contract", "../contract/out");
    build_docs_spa();
    let html =
        std::fs::read_to_string("../out/vastrum-docs.html").expect("Failed to read SPA HTML");
    let brotli_html_content =
        vastrum_shared_types::compression::brotli::brotli_compress_html(&html);

    let admin_key = vastrum_shared_types::crypto::ed25519::PrivateKey::from_rng();
    let admin_pub = admin_key.public_key();

    let client =
        ContractAbiClient::deploy("../contract/out/contract.wasm", brotli_html_content, admin_pub)
            .await;

    let site_id = client.site_id();
    register_domain(site_id, "docs").await.await_confirmation().await;

    println!();
    println!("Deploy complete");
    println!("site_id: {site_id}");
    println!("admin_key: {admin_key}");
}
