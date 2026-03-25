//never roll your own crypto ;)
//https://web.archive.org/web/20220325094144/https://codahale.com/towards-a-safer-footgun/
//probably should separate encryption keys from signing keys
#[derive(BorshSerialize, BorshDeserialize, Debug, Serialize, Deserialize)]
pub struct CipherText {
    pub cipher_text: cha_cha20_poly1305::CipherText,
    pub from_pub_key: x25519::PublicKey,
}

pub fn encrypt_bytes_ed25519(
    data: &[u8],
    private_key: &ed25519::PrivateKey,
    target_pub_key: x25519::PublicKey,
) -> CipherText {
    let x25519_private_key = x25519::PrivateKey::from_ed25519(private_key);
    return encrypt_bytes_x25519(data, &x25519_private_key, target_pub_key);
}
pub fn encrypt_bytes_x25519(
    data: &[u8],
    private_key: &x25519::PrivateKey,
    target_pub_key: x25519::PublicKey,
) -> CipherText {
    let shared = private_key.diffie_hellman(target_pub_key);
    let key_bytes = derive_key(&shared);

    let encrypter = cha_cha20_poly1305::ChaCha20Poly1305::from(key_bytes);
    let cipher_text = encrypter.encrypt(data);

    return CipherText { cipher_text, from_pub_key: private_key.public_key() };
}

pub fn decrypt_bytes_ed25519(
    cipher_text: CipherText,
    private_key: &ed25519::PrivateKey,
) -> Option<Vec<u8>> {
    let x25519_private_key = x25519::PrivateKey::from_ed25519(private_key);
    return decrypt_bytes_x25519(cipher_text, &x25519_private_key);
}

pub fn decrypt_bytes_x25519(
    cipher_text: CipherText,
    private_key: &x25519::PrivateKey,
) -> Option<Vec<u8>> {
    let shared = private_key.diffie_hellman(cipher_text.from_pub_key);
    let key_bytes = derive_key(&shared);
    let decryptor = cha_cha20_poly1305::ChaCha20Poly1305::from(key_bytes);
    let decrypted = decryptor.decrypt(cipher_text.cipher_text);
    return decrypted;
}

pub fn decrypt_string_x25519(cipher_text: CipherText, private_key: &x25519::PrivateKey) -> String {
    let bytes = decrypt_bytes_x25519(cipher_text, private_key);

    if let Some(bytes) = bytes {
        let string_value = String::decode(&bytes).unwrap();
        return string_value;
    } else {
        return "".to_string();
    }
}
pub fn encrypt_string_x25519(
    data: &str,
    private_key: &x25519::PrivateKey,
    target_pub_key: x25519::PublicKey,
) -> CipherText {
    let bytes = borsh::to_vec(data).unwrap();
    return encrypt_bytes_x25519(&bytes, private_key, target_pub_key);
}

///TODO is correct?
fn derive_key(shared: &[u8]) -> [u8; 32] {
    let hk = Hkdf::<Sha256>::new(None, shared);
    let mut okm = [0u8; 32];
    hk.expand(b"ed25519-x25519-chacha20poly1305", &mut okm).unwrap();
    return okm;
}

#[allow(unused_imports)]
use crate::borsh::*;
use crate::crypto::{cha_cha20_poly1305, ed25519, x25519};
use borsh::{BorshDeserialize, BorshSerialize};
use hkdf::Hkdf;
use serde::{Deserialize, Serialize};
use sha2::Sha256;

#[cfg(test)]
mod tests {
    use crate::crypto::{
        ed25519,
        encryption::{decrypt_bytes_ed25519, encrypt_bytes_ed25519},
        x25519,
    };

    #[test]
    fn test_ed25519_to_x25519_public_key_consistency() {
        let ed_sk = ed25519::PrivateKey::from_rng();
        let ed_pk = ed_sk.public_key();
        let x_pk_from_private = x25519::PrivateKey::from_ed25519(&ed_sk).public_key();
        let x_pk_from_public = x25519::PublicKey::from_ed25519_public_key(&ed_pk).unwrap();
        assert_eq!(x_pk_from_private.to_bytes(), x_pk_from_public.to_bytes());
    }

    #[test]
    fn test_simple_encryption() {
        let alice_kp = ed25519::PrivateKey::from_rng();
        let bob_kp = ed25519::PrivateKey::from_rng();
        let bob_x_pub = x25519::PrivateKey::from_ed25519(&bob_kp).public_key();
        let input = b"Hello world".to_vec();
        let ciphertext = encrypt_bytes_ed25519(&input, &alice_kp, bob_x_pub);
        let decrypted = decrypt_bytes_ed25519(ciphertext, &bob_kp).unwrap();

        assert_eq!(input, decrypted);
    }
}
