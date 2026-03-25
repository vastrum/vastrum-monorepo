use concourse_abi::*;
use vastrum_native_lib::deployers::{
    build::{build_contract, run},
    deploy::register_domain,
};
use vastrum_shared_types::crypto::ed25519;

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
    let private_key = ed25519::PrivateKey::try_from_string(
        "95e8d68353232e5e4fefcf645dc08eb53d01c8f01cd5b07e0c2245672ee176c5".to_string(),
    )
    .expect("invalid private key hex");
    let moderator = private_key.public_key();
    let client =
        ContractAbiClient::deploy("../contract/out/contract.wasm", brotli_html_content, moderator)
            .await;

    let client = client.with_account_key(private_key);

    register_domain(client.site_id(), "concourse").await.await_confirmation().await;
    register_domain(client.site_id(), "index").await.await_confirmation().await;

    client.create_category("General discussions", "").await.await_confirmation().await;
    client.create_category("Site development on vastrum", "").await.await_confirmation().await;
    client.create_category("Vastrum technical proposals", "").await.await_confirmation().await;
}
