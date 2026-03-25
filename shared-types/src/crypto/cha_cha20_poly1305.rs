#[derive(Serialize, Deserialize, BorshSerialize, BorshDeserialize, Clone, Debug)]
pub struct ChaCha20Poly1305 {
    key: [u8; 32],
}
impl From<[u8; 32]> for ChaCha20Poly1305 {
    fn from(bytes: [u8; 32]) -> Self {
        ChaCha20Poly1305 { key: bytes }
    }
}
impl ChaCha20Poly1305 {
    pub fn to_bytes(&self) -> &[u8; 32] {
        return &self.key;
    }
    pub fn from_rng() -> Self {
        let mut bytes = [0u8; 32];
        rand::fill(&mut bytes);
        return ChaCha20Poly1305::from(bytes);
    }
    pub fn encrypt(&self, data: &[u8]) -> CipherText {
        let encryptor = chacha20poly1305::ChaCha20Poly1305::new(Key::from_slice(&self.key));

        let mut nonce_bytes = [0u8; 12];
        rand::fill(&mut nonce_bytes);
        let nonce = chacha20poly1305::Nonce::from_slice(&nonce_bytes);

        let encrypted_data = encryptor.encrypt(nonce, data.as_ref()).unwrap();
        return CipherText { nonce_bytes, encrypted_data };
    }
    pub fn decrypt(&self, cipher_text: CipherText) -> Option<Vec<u8>> {
        let decryptor = chacha20poly1305::ChaCha20Poly1305::new(Key::from_slice(&self.key));

        let decrypted = decryptor
            .decrypt(
                Nonce::from_slice(&cipher_text.nonce_bytes),
                cipher_text.encrypted_data.as_ref(),
            )
            .ok()?;
        return Some(decrypted);
    }
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, Serialize, Deserialize)]
pub struct CipherText {
    pub nonce_bytes: [u8; 12],
    pub encrypted_data: Vec<u8>,
}

#[allow(unused_imports)]
use crate::borsh::*;
use borsh::BorshDeserialize;
use borsh::BorshSerialize;
use chacha20poly1305::Key;
use chacha20poly1305::KeyInit;
use chacha20poly1305::Nonce;
use chacha20poly1305::aead::Aead;
use serde::Deserialize;
use serde::Serialize;
