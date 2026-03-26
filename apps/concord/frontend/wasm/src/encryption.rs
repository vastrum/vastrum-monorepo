use vastrum_shared_types::borsh::BorshExt;
use vastrum_shared_types::crypto::cha_cha20_poly1305::{ChaCha20Poly1305, CipherText};
use vastrum_shared_types::crypto::ed25519::PublicKey;
use vastrum_shared_types::crypto::sha256::sha256_hash;
use vastrum_shared_types::crypto::x25519;
use vastrum_frontend_lib::get_pub_key;

pub async fn get_personal_key() -> [u8; 32] {
    let salt = vastrum_frontend_lib::get_private_salt("concord_server_keys".to_string()).await;
    let key = salt.to_bytes();
    return key;
}

pub async fn load_server_key(server_id: u64) -> Option<[u8; 32]> {
    let client = crate::new_client();
    let public_key = get_pub_key().await;
    let state = client.state().await;
    let profile = state.user_profiles.get(&public_key).await?;
    let (_, encrypted_bytes) =
        profile.encrypted_server_keys.iter().find(|(id, _)| *id == server_id)?;
    let ciphertext = CipherText::decode(encrypted_bytes).ok()?;
    let personal_key = get_personal_key().await;
    let cipher = ChaCha20Poly1305::from(personal_key);
    let decrypted = cipher.decrypt(ciphertext)?;
    let mut key = [0u8; 32];
    key.copy_from_slice(&decrypted);
    return Some(key);
}

pub async fn encrypt_server_key(server_key: &[u8; 32]) -> [u8; 64] {
    let personal_key = get_personal_key().await;
    let cipher = ChaCha20Poly1305::from(personal_key);
    let ciphertext = cipher.encrypt(server_key);
    let encoded: [u8; 64] = ciphertext.encode().try_into().unwrap();
    return encoded;
}

pub fn try_decrypt_content(cipher: &ChaCha20Poly1305, content: &str) -> String {
    let decoded_bytes = match hex::decode(content) {
        Ok(decoded) => decoded,
        Err(_) => return content.to_string(),
    };
    let ciphertext = match CipherText::decode(&decoded_bytes) {
        Ok(parsed) => parsed,
        Err(_) => return content.to_string(),
    };
    let decrypted = match cipher.decrypt(ciphertext) {
        Some(plaintext) => String::from_utf8_lossy(&plaintext).to_string(),
        None => content.to_string(),
    };
    return decrypted;
}

pub async fn get_dm_cipher(partner: &PublicKey) -> Option<ChaCha20Poly1305> {
    let my_private = vastrum_frontend_lib::get_private_key().await;
    let my_x25519_private = x25519::PrivateKey::from_ed25519(&my_private);
    let partner_x25519_pub = x25519::PublicKey::from_ed25519_public_key(partner)?;
    let shared_secret = my_x25519_private.diffie_hellman(partner_x25519_pub);
    let key = sha256_hash(&shared_secret).to_bytes();
    Some(ChaCha20Poly1305::from(key))
}
