use mapper_abi::*;
use mapper_tile_uploader::*;
use vastrum_shared_types::crypto::ed25519::PrivateKey;
use vastrum_shared_types::crypto::sha256::Sha256Digest;

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 4 {
        eprintln!("Usage: mapper-tile-uploader <site-id> <admin-key-hex> <mbtiles-path>");
        std::process::exit(1);
    }

    let site_id = Sha256Digest::from_string(&args[1]).expect("Invalid site_id (expected base32)");
    let admin_key =
        PrivateKey::try_from_string(args[2].clone()).expect("Invalid admin_key (expected hex)");
    let mbtiles_path = &args[3];

    let client = ContractAbiClient::new(site_id).with_account_key(admin_key);
    let checkpoint_path = format!("{mbtiles_path}.progress");

    upload_tiles(&client, mbtiles_path, &checkpoint_path).await;
}
