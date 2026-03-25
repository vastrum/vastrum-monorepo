use chatter_abi::*;
use vastrum_native_lib::deployers::{
    build::{build_contract, run},
    deploy::register_domain,
};

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
    let brotli_html_content = vastrum_shared_types::compression::brotli::brotli_compress_html(&html);
    let client =
        ContractAbiClient::deploy("../contract/out/contract.wasm", brotli_html_content).await;
    register_domain(client.site_id(), "chatter").await.await_confirmation().await;
    register_domain(client.site_id(), "index").await.await_confirmation().await;
}
