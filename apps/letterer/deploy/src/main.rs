use letterer_abi::*;
use vastrum_native_lib::deployers::{
    build::{build_contract, run},
    deploy::register_domain,
};
use vastrum_shared_types::crypto::sha256::Sha256Digest;

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
    let client =
        ContractAbiClient::deploy("../contract/out/contract.wasm", brotli_html_content).await;
    register_domain(client.site_id(), "letterer").await.await_confirmation().await;
    register_domain(client.site_id(), "index").await.await_confirmation().await;
    register_domain(client.site_id(), client.site_id().to_string())
        .await
        .await_confirmation()
        .await;
    // static testnet site_id registration in case of network redeployment
    // causing site to have different site_id and causing dead links
    let static_site_id =
        Sha256Digest::from_string("yozq5azfm26qi3vceclwz57fg2727yhqi6ccha5khhnp2uepqj7a").unwrap();
    register_domain(static_site_id, static_site_id.to_string()).await.await_confirmation().await;
}
