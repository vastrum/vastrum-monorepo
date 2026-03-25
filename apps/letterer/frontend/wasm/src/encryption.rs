#[derive(BorshSerialize, BorshDeserialize, Default, Clone)]
pub struct DocumentKeychain {
    pub document_keys: HashMap<PublicKey, DocumentAccessKey>,
}

#[derive(BorshSerialize, BorshDeserialize, Clone)]
pub struct DocumentAccessKey {
    pub document_private_key: PrivateKey,
    pub created_by_me: bool,
}

#[derive(BorshSerialize, BorshDeserialize, Clone)]
pub struct DocumentMeta {
    pub title: String,
    pub created_at: u64,
    pub last_modified: u64,
}

#[derive(BorshSerialize, BorshDeserialize, Clone)]
pub struct DocumentContent {
    pub content: String,
}

pub async fn get_personal_key() -> ChaCha20Poly1305 {
    let salt = get_private_salt("letterer_document_keys".to_string()).await;
    return ChaCha20Poly1305::from(salt.to_bytes());
}

pub fn derive_symmetric_key(priv_key: &PrivateKey) -> ChaCha20Poly1305 {
    let hash = sha256_hash(priv_key.to_bytes());
    return ChaCha20Poly1305::from(hash.to_bytes());
}

pub fn encrypt_content(doc_priv: &PrivateKey, content: &DocumentContent) -> Vec<u8> {
    let serialized = content.encode();
    let doc_cipher = derive_symmetric_key(doc_priv);
    let ciphertext = doc_cipher.encrypt(&serialized);
    return ciphertext.encode();
}

pub fn decrypt_content(doc_priv: &PrivateKey, encrypted: &[u8]) -> Option<DocumentContent> {
    let doc_cipher = derive_symmetric_key(doc_priv);
    let ciphertext = CipherText::decode(encrypted).ok()?;
    let decrypted = doc_cipher.decrypt(ciphertext)?;
    return DocumentContent::decode(&decrypted).ok();
}

pub fn encrypt_metadata(doc_priv: &PrivateKey, meta: &DocumentMeta) -> Vec<u8> {
    let serialized = meta.encode();
    let doc_cipher = derive_symmetric_key(doc_priv);
    let ciphertext = doc_cipher.encrypt(&serialized);
    return ciphertext.encode();
}

pub fn decrypt_metadata(doc_priv: &PrivateKey, encrypted: &[u8]) -> Option<DocumentMeta> {
    let doc_cipher = derive_symmetric_key(doc_priv);
    let ciphertext = CipherText::decode(encrypted).ok()?;
    let decrypted = doc_cipher.decrypt(ciphertext)?;
    return DocumentMeta::decode(&decrypted).ok();
}

pub fn sign_document(doc_priv: &PrivateKey, operation: &DocumentWriteOperation) -> Signature {
    let encoded = borsh::to_vec(operation).unwrap();
    let hash = sha256_hash(&encoded);
    return doc_priv.sign(&hash.to_bytes());
}

pub fn encrypt_keychain(index: &DocumentKeychain, cipher: &ChaCha20Poly1305) -> Vec<u8> {
    let data = index.encode();
    let ciphertext = cipher.encrypt(&data);
    return ciphertext.encode();
}

pub fn decrypt_keychain(encrypted: &[u8], cipher: &ChaCha20Poly1305) -> Option<DocumentKeychain> {
    let ciphertext = CipherText::decode(encrypted).ok()?;
    let decrypted = cipher.decrypt(ciphertext)?;
    return DocumentKeychain::decode(&decrypted).ok();
}

pub async fn load_keychain() -> DocumentKeychain {
    let client = new_client();
    let public_key = get_pub_key().await;
    let cipher = get_personal_key().await;
    let state = client.state().await;
    return match state.user_data.get(&public_key).await {
        Some(encrypted) => decrypt_keychain(&encrypted, &cipher).unwrap_or_default(),
        None => DocumentKeychain::default(),
    };
}

pub fn parse_pub_key(hex: &str) -> Option<PublicKey> {
    let decoded = hex::decode(hex).ok()?;
    let bytes: [u8; 32] = decoded.try_into().ok()?;
    return PublicKey::try_from_bytes(bytes);
}

pub async fn get_doc_private_key(doc_pub: &PublicKey) -> Option<PrivateKey> {
    let index = load_keychain().await;
    let entry = index.document_keys.get(doc_pub)?;
    return Some(entry.document_private_key.clone());
}

pub async fn save_keychain(index: &DocumentKeychain) -> String {
    let cipher = get_personal_key().await;
    let encrypted = encrypt_keychain(index, &cipher);
    let client = new_client();
    let sent_tx = client.save_user_data(encrypted).await;
    return sent_tx.tx_hash().to_string();
}

pub async fn save_document(
    doc_priv: &PrivateKey,
    content: &DocumentContent,
    meta: &DocumentMeta,
) -> String {
    let encrypted_content = encrypt_content(doc_priv, content);
    let encrypted_meta = encrypt_metadata(doc_priv, meta);
    let operation = DocumentWriteOperation { content: encrypted_content, metadata: encrypted_meta };
    let signature = sign_document(doc_priv, &operation);
    let doc_pub = doc_priv.public_key();
    let client = new_client();
    let sent_tx = client.save_document(doc_pub, signature, operation).await;
    return sent_tx.tx_hash().to_string();
}

use crate::new_client;
use borsh::{BorshDeserialize, BorshSerialize};
use letterer_abi::{DocumentWriteOperation, SentTxBehavior};
use vastrum_shared_types::borsh::BorshExt;
use vastrum_shared_types::crypto::cha_cha20_poly1305::{ChaCha20Poly1305, CipherText};
use vastrum_shared_types::crypto::ed25519::{PrivateKey, PublicKey, Signature};
use vastrum_shared_types::crypto::sha256::sha256_hash;
use std::collections::HashMap;
use vastrum_frontend_lib::{get_private_salt, get_pub_key};
