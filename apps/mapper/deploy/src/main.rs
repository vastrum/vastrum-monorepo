mod tiles;

use mapper_abi::*;
use vastrum_native_lib::deployers::build::{build_contract, run};
use vastrum_native_lib::deployers::deploy::register_domain;
use vastrum_shared_types::crypto::ed25519::PrivateKey;
use vastrum_shared_types::crypto::sha256::Sha256Digest;

const KEYS_FILE: &str = "mapper-deploy.keys";
const MAPPER_DOMAIN: &str = "mapper";

const STATIC_SITE_ID: &str = "lzdtxcpp6ivwje55o74dugj7f4vie6qzrsp6kybqyi7ofo3yt75q";

struct Deployment {
    client: ContractAbiClient,
    site_id: Sha256Digest,
    admin_key: PrivateKey,
}

async fn deploy() -> Deployment {
    let html =
        std::fs::read_to_string("../frontend/dist/index.html").expect("Failed to read HTML file");
    let brotli_html_content =
        vastrum_shared_types::compression::brotli::brotli_compress_html(&html);

    let admin_key = PrivateKey::from_rng();
    let client = ContractAbiClient::deploy(
        "../contract/out/contract.wasm",
        brotli_html_content,
        admin_key.public_key(),
    )
    .await;
    let client = client.with_account_key(admin_key.clone());
    let site_id = client.site_id();

    std::fs::write(KEYS_FILE, format!("{site_id}\n{admin_key}\n"))
        .expect("failed to write keys file");
    println!("deploy identity saved to {KEYS_FILE}");

    register_domains(site_id).await;
    return Deployment { client, site_id, admin_key };
}

async fn register_domains(site_id: Sha256Digest) {
    register_domain(site_id, MAPPER_DOMAIN).await.await_confirmation().await;
    register_domain(site_id, "index").await.await_confirmation().await;
    register_domain(site_id, site_id.to_string()).await.await_confirmation().await;
    let static_id = Sha256Digest::from_string(STATIC_SITE_ID).unwrap();
    register_domain(site_id, static_id.to_string()).await.await_confirmation().await;
}

fn build_frontend() {
    run("npm install", "../frontend");
    let script = if cfg!(debug_assertions) { "build" } else { "build:prod" };
    run(&format!("npm run {script}"), "../frontend");
}

#[tokio::main]
async fn main() {
    build_contract("../contract", "../contract/out");
    build_frontend();

    let deployment = deploy().await;
    let client = std::sync::Arc::new(deployment.client);

    let planet = std::env::args().any(|a| a == "--planet");
    let mbtiles_path = tiles::ensure_tiles(planet);

    let checkpoint_path = format!("{mbtiles_path}.{}.progress", deployment.site_id);
    mapper_tile_uploader::upload_tiles(client.clone(), &mbtiles_path, &checkpoint_path)
        .await
        .expect("tile upload failed");

    if planet {
        mapper_tile_uploader::places::build_and_upload_search_index(
            tiles::PLANET_PBF,
            Some(mbtiles_path.as_str()),
            client,
        )
        .await
        .expect("search index upload failed");
    }

    println!();
    println!("=== Deploy complete ===");
    println!("site_id: {}", deployment.site_id);
    println!("admin_key: {}  (also saved in {})", deployment.admin_key, KEYS_FILE);
}
