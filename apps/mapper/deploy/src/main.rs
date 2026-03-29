const MAPPER_DOMAIN: &str = "mapper";

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

    let admin_key = vastrum_shared_types::crypto::ed25519::PrivateKey::from_rng();
    let admin_pub = admin_key.public_key();

    let client =
        ContractAbiClient::deploy("../contract/out/contract.wasm", brotli_html_content, admin_pub)
            .await;
    let client = client.with_account_key(admin_key.clone());

    let site_id = client.site_id();
    register_domain(site_id, MAPPER_DOMAIN).await.await_confirmation().await;
    register_domain(site_id, "index").await.await_confirmation().await;
    register_domain(site_id, site_id.to_string()).await.await_confirmation().await;
    // static testnet site_id registration in case of network redeployment
    // causing site to have different site_id and causing dead links
    let static_site_id = vastrum_shared_types::crypto::sha256::Sha256Digest::from_string(
        "lzdtxcpp6ivwje55o74dugj7f4vie6qzrsp6kybqyi7ofo3yt75q",
    )
    .unwrap();
    register_domain(static_site_id, static_site_id.to_string()).await.await_confirmation().await;

    // Always upload monaco tiles
    let mbtiles_path = "../tiles/output.mbtiles";
    if !std::path::Path::new(mbtiles_path).exists() {
        println!("No mbtiles found, generating Monaco tiles...");
        run("./generate-tiles.sh", "../tiles");
    }
    let checkpoint_path = format!("{mbtiles_path}.progress");
    mapper_tile_uploader::upload_tiles(&client, mbtiles_path, &checkpoint_path).await;

    println!();
    println!("=== Deploy complete ===");
    println!("site_id: {site_id}");
    println!("admin_key: {admin_key}");
    println!();
    println!("To upload tiles from a different mbtiles file:");
    println!("  cargo run -p mapper-tile-uploader -- {site_id} {admin_key} <mbtiles-path>");
}

use mapper_abi::*;
use vastrum_native_lib::deployers::{
    build::{build_contract, run},
    deploy::register_domain,
};
use vastrum_shared_types;
