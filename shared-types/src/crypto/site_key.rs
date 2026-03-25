use super::ed25519::PrivateKey;
use super::sha256::{Sha256Digest, sha256_hash};
use crate::borsh::BorshExt;

pub fn derive_site_key(master_key: &PrivateKey, site_id: Sha256Digest) -> PrivateKey {
    let bytes = [site_id.encode(), b"VASTRUM_PRIVATE_SALT_NAMESPACE".to_vec()].concat();
    let signature = master_key.sign(&bytes);
    let salt = sha256_hash(&signature.encode());
    return PrivateKey::from_bytes(salt.to_bytes());
}
