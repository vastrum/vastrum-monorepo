#[wasm_bindgen]
pub async fn create_document(title: String) -> String {
    let document_private_key = PrivateKey::from_rng();
    let document_pub_key = document_private_key.public_key();
    let mut keychain = load_keychain().await;
    keychain.document_keys.insert(
        document_pub_key,
        DocumentAccessKey {
            document_private_key: document_private_key.clone(),
            created_by_me: true,
        },
    );
    let now = now_secs();
    let meta = DocumentMeta { title, created_at: now, last_modified: now };
    let content = DocumentContent { content: String::new() };
    save_keychain(&keychain).await;
    let tx_hash = save_document(&document_private_key, &content, &meta).await;
    return tx_hash;
}

#[wasm_bindgen]
pub async fn join_document(doc_priv_hex: String) -> String {
    let key_bytes = hex::decode(&doc_priv_hex).unwrap();
    let private_key_bytes = <[u8; 32]>::try_from(key_bytes).unwrap();
    let document_private_key = PrivateKey::from_bytes(private_key_bytes);
    let document_pub_key = document_private_key.public_key();
    let mut index = load_keychain().await;
    index
        .document_keys
        .insert(document_pub_key, DocumentAccessKey { document_private_key, created_by_me: false });
    let tx_hash = save_keychain(&index).await;
    return tx_hash;
}

#[wasm_bindgen]
pub async fn save_content(doc_pub_hex: String, content: String) -> String {
    let doc_pub = parse_pub_key(&doc_pub_hex).unwrap();
    let (doc_priv, mut meta) = load_document_meta(&doc_pub).await;
    meta.last_modified = now_secs();
    let content = DocumentContent { content };
    let tx_hash = save_document(&doc_priv, &content, &meta).await;
    return tx_hash;
}

#[wasm_bindgen]
pub async fn rename_document(doc_pub_hex: String, new_title: String) -> String {
    let doc_pub = parse_pub_key(&doc_pub_hex).unwrap();
    let (doc_priv, mut meta) = load_document_meta(&doc_pub).await;
    meta.title = new_title;
    let client = new_client();
    let state = client.state().await;
    let existing_content = state.documents.get(&doc_pub).await.unwrap_or_default();
    let content = decrypt_content(&doc_priv, &existing_content).unwrap();
    let tx_hash = save_document(&doc_priv, &content, &meta).await;
    return tx_hash;
}

#[wasm_bindgen]
pub async fn delete_document(doc_pub_hex: String) -> String {
    let doc_pub = parse_pub_key(&doc_pub_hex).unwrap();
    let mut index = load_keychain().await;
    let entry = index.document_keys.get(&doc_pub).unwrap();
    let doc_priv = entry.document_private_key.clone();
    let meta = DocumentMeta { title: String::new(), created_at: 0, last_modified: 0 };
    let content = DocumentContent { content: String::new() };
    index.document_keys.remove(&doc_pub);
    save_keychain(&index).await;
    let tx_hash = save_document(&doc_priv, &content, &meta).await;
    return tx_hash;
}

#[wasm_bindgen]
pub async fn get_my_documents() -> Vec<JSDocumentMeta> {
    let index = load_keychain().await;
    let client = new_client();
    let state = client.state().await;
    let mut result = vec![];
    for (pub_key, e) in index.document_keys {
        let encrypted_meta = state.doc_metadata.get(&pub_key).await.unwrap();
        let meta = decrypt_metadata(&e.document_private_key, &encrypted_meta).unwrap();
        result.push(JSDocumentMeta {
            id: hex::encode(pub_key.to_bytes()),
            title: meta.title,
            created_at: meta.created_at,
            last_modified: meta.last_modified,
            created_by_me: e.created_by_me,
        });
    }
    return result;
}

#[wasm_bindgen]
pub async fn get_document_meta(doc_pub_hex: String) -> Option<JSDocumentMeta> {
    let doc_pub = parse_pub_key(&doc_pub_hex)?;
    let index = load_keychain().await;
    let entry = index.document_keys.get(&doc_pub)?;
    let doc_priv = entry.document_private_key.clone();
    let client = new_client();
    let state = client.state().await;
    let encrypted_meta = state.doc_metadata.get(&doc_pub).await.unwrap();
    let meta = decrypt_metadata(&doc_priv, &encrypted_meta).unwrap();
    return Some(JSDocumentMeta {
        id: doc_pub_hex,
        title: meta.title,
        created_at: meta.created_at,
        last_modified: meta.last_modified,
        created_by_me: entry.created_by_me,
    });
}

#[wasm_bindgen]
pub async fn get_document_content(doc_pub_hex: String) -> Option<String> {
    let doc_pub = parse_pub_key(&doc_pub_hex)?;
    let client = new_client();
    let state = client.state().await;
    let encrypted_bytes = state.documents.get(&doc_pub).await?;
    let doc_priv = get_doc_private_key(&doc_pub).await?;
    let doc = decrypt_content(&doc_priv, &encrypted_bytes)?;
    return Some(doc.content);
}

#[wasm_bindgen]
pub async fn get_document_key_hex(doc_pub_hex: String) -> Option<String> {
    let doc_pub = parse_pub_key(&doc_pub_hex)?;
    let doc_priv = get_doc_private_key(&doc_pub).await?;
    return Some(hex::encode(doc_priv.to_bytes()));
}


async fn load_document_meta(doc_pub: &PublicKey) -> (PrivateKey, DocumentMeta) {
    let index = load_keychain().await;
    let entry = index.document_keys.get(doc_pub).unwrap();
    let doc_priv = entry.document_private_key.clone();
    let client = new_client();
    let state = client.state().await;
    let encrypted_meta = state.doc_metadata.get(doc_pub).await.unwrap();
    let meta = decrypt_metadata(&doc_priv, &encrypted_meta).unwrap();
    return (doc_priv, meta);
}

pub fn new_client() -> ContractAbiClient {
    let client = ContractAbiClient::new(Sha256Digest::from([0u8; 32]));
    return client;
}

fn now_secs() -> u64 {
    return (js_sys::Date::now() / 1000.0) as u64;
}

#[derive(Serialize, Tsify)]
#[tsify(into_wasm_abi)]
pub struct JSDocumentMeta {
    pub id: String,
    pub title: String,
    pub created_at: u64,
    pub last_modified: u64,
    pub created_by_me: bool,
}

mod encryption;
use encryption::{
    DocumentAccessKey, DocumentContent, DocumentMeta, decrypt_content, decrypt_metadata,
    get_doc_private_key, load_keychain, parse_pub_key, save_document, save_keychain,
};
pub use letterer_abi::*;
use serde::Serialize;
use vastrum_shared_types::crypto::ed25519::{PrivateKey, PublicKey};
use vastrum_shared_types::crypto::sha256::Sha256Digest;
use tsify::Tsify;
use wasm_bindgen::prelude::*;
