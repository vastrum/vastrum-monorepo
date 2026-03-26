fn default_keystore_path() -> PathBuf {
    dirs::data_dir()
        .expect("could not determine data directory")
        .join("vastrum")
        .join("keystore.bin")
}

pub async fn start_node(keystore: Option<PathBuf>, rpc: bool) {
    let path = keystore.unwrap_or_else(default_keystore_path);
    vastrum_node::start_node_production(path, rpc).await;
}

pub fn generate_keys(output: PathBuf, wallet_key: String) -> Result<()> {
    let wallet_key =
        ed25519::PrivateKey::try_from_string(wallet_key).expect("invalid wallet key hex");
    let keystore = Keystore::from_wallet_key(&wallet_key);
    if let Some(parent) = output.parent() {
        std::fs::create_dir_all(parent)?;
    }
    keystore.save_to_file(&output);
    println!("Keystore saved to: {}", output.display());
    println!("Validator public key: {}", keystore.validator_private_key.public_key().to_string());
    println!("P2P public key:      {}", keystore.p2p_key.public_key().to_string());
    println!("DTLS fingerprint:    {}", keystore.dtls_key.fingerprint());
    return Ok(());
}

pub fn show_keys(keystore: PathBuf) {
    let ks = Keystore::load_from_file(&keystore);
    println!("Validator public key: {}", ks.validator_private_key.public_key().to_string());
    println!("P2P public key:      {}", ks.p2p_key.public_key().to_string());
    println!("DTLS fingerprint:    {}", ks.dtls_key.fingerprint());
}

use anyhow::Result;
use std::path::PathBuf;
use vastrum_node::keystore::keyset::Keystore;
use vastrum_shared_types::crypto::ed25519;
