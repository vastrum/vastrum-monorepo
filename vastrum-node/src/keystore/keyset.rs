pub fn insecure_generate_new_static_identity(seed: u64) -> Keystore {
    Keystore {
        validator_private_key: ed25519::PrivateKey::from_seed(seed),
        p2p_key: ed25519::PrivateKey::from_seed(seed + 1),
        dtls_key: vastrum_webrtc_direct_server::DtlsKey::from_seed(seed + 2),
    }
}

#[derive(Clone, Debug, BorshSerialize, BorshDeserialize)]
pub struct Keystore {
    pub validator_private_key: ed25519::PrivateKey,
    pub p2p_key: ed25519::PrivateKey,
    pub dtls_key: vastrum_webrtc_direct_server::DtlsKey,
}

impl Keystore {
    pub fn from_wallet_key(wallet_key: &ed25519::PrivateKey) -> Keystore {
        let hk = Hkdf::<Sha256>::new(None, wallet_key.to_bytes());

        let mut validator_bytes = [0u8; 32];
        hk.expand(b"vastrum-validator", &mut validator_bytes).unwrap();

        let mut p2p_bytes = [0u8; 32];
        hk.expand(b"vastrum-p2p", &mut p2p_bytes).unwrap();

        let mut dtls_bytes = [0u8; 32];
        hk.expand(b"vastrum-dtls", &mut dtls_bytes).unwrap();

        Keystore {
            validator_private_key: ed25519::PrivateKey::from_bytes(validator_bytes),
            p2p_key: ed25519::PrivateKey::from_bytes(p2p_bytes),
            dtls_key: vastrum_webrtc_direct_server::DtlsKey::from_bytes(dtls_bytes),
        }
    }

    pub fn generate() -> Keystore {
        Keystore {
            validator_private_key: ed25519::PrivateKey::from_rng(),
            p2p_key: ed25519::PrivateKey::from_rng(),
            dtls_key: vastrum_webrtc_direct_server::DtlsKey::from_rng(),
        }
    }

    pub fn save_to_file(&self, path: &Path) {
        std::fs::write(path, self.encode()).unwrap();
    }

    pub fn load_from_file(path: &Path) -> Keystore {
        let bytes = std::fs::read(path).unwrap();
        return Keystore::decode(&bytes).unwrap();
    }

    pub fn load_or_create(path: &Path) -> Keystore {
        if path.exists() {
            return Keystore::load_from_file(path);
        } else {
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent).unwrap();
            }
            let keystore = Keystore::generate();
            keystore.save_to_file(path);
            return keystore;
        }
    }
}

use borsh::{BorshDeserialize, BorshSerialize};
use hkdf::Hkdf;
use sha2::Sha256;
use vastrum_shared_types::borsh::BorshExt;
use vastrum_shared_types::crypto::ed25519;
use std::path::Path;
