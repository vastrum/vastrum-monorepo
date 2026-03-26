const NONCE_LEN: usize = 12;

pub struct TransportCipher {
    cipher: chacha20poly1305::ChaCha20Poly1305,
}
impl TransportCipher {
    pub fn new(key: [u8; 32]) -> Self {
        let cipher = chacha20poly1305::ChaCha20Poly1305::new(Key::from_slice(&key));
        TransportCipher { cipher }
    }

    pub fn from_shared_secret(shared_secret: &[u8; 32], is_initiator: bool) -> (Self, Self) {
        let hk = Hkdf::<Sha256>::new(None, shared_secret);

        let mut initiator_to_responder_key = [0u8; 32];
        hk.expand(b"initiator-to-responder", &mut initiator_to_responder_key).unwrap();

        let mut responder_to_initiator_key = [0u8; 32];
        hk.expand(b"responder-to-initiator", &mut responder_to_initiator_key).unwrap();

        let send_cipher;
        let recv_cipher;
        if is_initiator {
            send_cipher = TransportCipher::new(initiator_to_responder_key);
            recv_cipher = TransportCipher::new(responder_to_initiator_key);
        } else {
            send_cipher = TransportCipher::new(responder_to_initiator_key);
            recv_cipher = TransportCipher::new(initiator_to_responder_key);
        }

        (send_cipher, recv_cipher)
    }

    pub fn encrypt(&self, plaintext: &[u8]) -> Vec<u8> {
        let mut nonce_bytes = [0u8; NONCE_LEN];
        rng::fill_bytes(&mut nonce_bytes);

        let nonce = chacha20poly1305::Nonce::from_slice(&nonce_bytes);
        let ciphertext = self.cipher.encrypt(nonce, plaintext).unwrap();

        let mut encrypted = Vec::with_capacity(NONCE_LEN + ciphertext.len());
        encrypted.extend_from_slice(&nonce_bytes);
        encrypted.extend_from_slice(&ciphertext);
        return encrypted;
    }

    pub fn decrypt(&self, data: &[u8]) -> eyre::Result<Vec<u8>> {
        if data.len() < NONCE_LEN {
            return Err(eyre::eyre!("frame too short"));
        }
        let (nonce_bytes, ciphertext) = data.split_at(NONCE_LEN);
        let nonce = chacha20poly1305::Nonce::from_slice(nonce_bytes);

        self.cipher.decrypt(nonce, ciphertext).map_err(|_| eyre::eyre!("decryption failed"))
    }
}

use crate::utils::rng;
use chacha20poly1305::Key;
use chacha20poly1305::KeyInit;
use chacha20poly1305::aead::Aead;
use hkdf::Hkdf;
use sha2::Sha256;
