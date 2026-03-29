use concourse_abi::*;
use vastrum_native_lib::deployers::{
    build::{build_contract, run},
    deploy::register_domain,
};
use vastrum_shared_types::crypto::{ed25519, sha256::Sha256Digest};

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

    let html =
        std::fs::read_to_string("../frontend/dist/index.html").expect("Failed to read HTML file");
    let brotli_html_content =
        vastrum_shared_types::compression::brotli::brotli_compress_html(&html);
    let admin_key = ed25519::PrivateKey::from_rng();
    let moderator = admin_key.public_key();
    let client =
        ContractAbiClient::deploy("../contract/out/contract.wasm", brotli_html_content, moderator)
            .await;

    let client = client.with_account_key(admin_key.clone());

    register_domain(client.site_id(), "concourse").await.await_confirmation().await;
    register_domain(client.site_id(), "index").await.await_confirmation().await;
    register_domain(client.site_id(), client.site_id().to_string())
        .await
        .await_confirmation()
        .await;
    // static testnet site_id registration in case of network redeployment
    // causing site to have different site_id and causing dead links
    let static_site_id =
        Sha256Digest::from_string("xv6edwxtsjtlujz2z7hgbkkshwx2im6bbvx3nxzyczumsnhwddrq").unwrap();
    register_domain(static_site_id, static_site_id.to_string()).await.await_confirmation().await;

    client.create_category("General discussions", "").await.await_confirmation().await;
    client.create_category("Site development on vastrum", "").await.await_confirmation().await;
    client.create_category("Vastrum technical proposals", "").await.await_confirmation().await;

    println!();
    println!("=== Deploy complete ===");
    println!("site_id: {}", client.site_id());
    println!("admin_key: {admin_key}");
}
