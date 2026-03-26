mod dm;
mod encryption;
mod server;
mod types;
mod user;

pub use concord_abi::*;
pub use types::*;

use vastrum_shared_types::crypto::ed25519::PublicKey;
use vastrum_shared_types::crypto::sha256::Sha256Digest;

pub fn string_to_pubkey(hex_str: &str) -> PublicKey {
    let decoded_bytes = hex::decode(hex_str).unwrap_or_else(|_| vec![0u8; 32]);
    let mut key_bytes = [0u8; 32];
    key_bytes.copy_from_slice(&decoded_bytes[..32]);
    let pubkey = PublicKey::try_from_bytes(key_bytes).unwrap();
    return pubkey;
}

pub fn new_client() -> ContractAbiClient {
    let client = ContractAbiClient::new(Sha256Digest::from([0u8; 32]));
    return client;
}

pub fn make_dm_key(a: PublicKey, b: PublicKey) -> DmKey {
    if a.to_bytes() < b.to_bytes() {
        DmKey { user_a: a, user_b: b }
    } else {
        DmKey { user_a: b, user_b: a }
    }
}

pub fn dm_other(key: &DmKey, me: PublicKey) -> PublicKey {
    if key.user_a == me { key.user_b } else { key.user_a }
}

