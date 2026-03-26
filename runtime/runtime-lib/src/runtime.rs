use vastrum_runtime_shared::{
    Ed25519PublicKey, Ed25519Signature, GetMessageSenderResponse, KeyValueInsertCall,
    KeyValueReadCall, KeyValueReadResponse, LogCall, RegisterStaticRouteCall,
};
use vastrum_bindings_guest::runtime_raw;

/// Get the message senders public key.
pub fn message_sender() -> Ed25519PublicKey {
    let bytes = runtime_raw::message_sender();
    let response: GetMessageSenderResponse = borsh::from_slice(&bytes).unwrap();
    let message_sender = response.sender;
    return message_sender;
}

/// Get the current block time.
pub fn block_time() -> u64 {
    let block_time = runtime_raw::block_time();
    return block_time;
}

/// Register a static route with brotli compressed HTML content.
pub fn register_static_route(route: &str, content: &[u8]) {
    let route = route.to_string();
    let brotli_html_content = content.to_vec();
    let args = RegisterStaticRouteCall { route, brotli_html_content };
    runtime_raw::register_static_route(&borsh::to_vec(&args).unwrap());
}

/// Insert a keyvalue pair into storage.
pub fn kv_insert(key: &str, value: &[u8]) {
    let args = KeyValueInsertCall { key: key.to_string(), value: value.to_vec() };
    runtime_raw::kv_insert(&borsh::to_vec(&args).unwrap());
}

/// Delete a key from storage.
pub fn kv_delete(key: &str) {
    //zero content vec insertion means it will the value will be pruned from the database
    kv_insert(key, &[]);
}

/// Read a value from storage by key.
pub fn kv_get(key: &str) -> Vec<u8> {
    let args = KeyValueReadCall { key: key.to_string() };
    let bytes = runtime_raw::kv_get(&borsh::to_vec(&args).unwrap());
    if bytes.is_empty() {
        return Vec::new();
    }
    let response: KeyValueReadResponse = borsh::from_slice(&bytes).unwrap();
    let value = response.value;
    return value;
}

/// Log debug message
pub fn log(message: &str) {
    let args = LogCall { message: message.to_string() };
    runtime_raw::log(&borsh::to_vec(&args).unwrap());
}

/// Verify an Ed25519 signature
pub fn verify_ed25519(pub_key: &Ed25519PublicKey, msg: &[u8], sig: &Ed25519Signature) -> bool {
    let Ok(pk) = ed25519_compact::PublicKey::from_slice(&pub_key.bytes) else {
        return false;
    };
    let Ok(signature) = ed25519_compact::Signature::from_slice(&sig.bytes) else {
        return false;
    };
    pk.verify(msg, &signature).is_ok()
}

pub trait Ed25519Verify {
    fn verify(&self, msg: &[u8], sig: &Ed25519Signature) -> bool;
}

impl Ed25519Verify for Ed25519PublicKey {
    fn verify(&self, msg: &[u8], sig: &Ed25519Signature) -> bool {
        verify_ed25519(self, msg, sig)
    }
}

/// Compute SHA256 hash of data.
pub fn sha256(data: &[u8]) -> [u8; 32] {
    use sha2::{Digest, Sha256};
    Sha256::digest(data).into()
}

/// Get next unique nonce for KvMap/KvVec namespace allocation.
pub fn next_nonce() -> u64 {
    let bytes = kv_get("__nonce__");
    let n: u64 = borsh::from_slice(&bytes).unwrap_or(0);
    kv_insert("__nonce__", &borsh::to_vec(&(n + 1)).unwrap());
    let next_nonce = n + 1;
    return next_nonce;
}
