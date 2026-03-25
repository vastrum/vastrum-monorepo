pub struct HostState {
    pub site_id: Sha256Digest,
    pub message_sender: ed25519::PublicKey,
    pub block_timestamp: u64,
    pub limits: StoreLimits,
    pub db: Arc<BatchDb>,
}

impl HostState {
    pub fn new(
        site_id: Sha256Digest,
        message_sender: ed25519::PublicKey,
        block_timestamp: u64,
        limits: StoreLimits,
        db: Arc<BatchDb>,
    ) -> HostState {
        HostState { site_id, message_sender, block_timestamp, limits, db }
    }
}

impl HostRuntime for HostState {
    fn message_sender(&self) -> Vec<u8> {
        let sender: Ed25519PublicKey = self.message_sender.into();

        let response = GetMessageSenderResponse { sender };
        return response.encode();
    }

    fn block_time(&self) -> u64 {
        return self.block_timestamp;
    }

    fn register_static_route(&mut self, args: &[u8]) {
        let Ok(RegisterStaticRouteCall { route, brotli_html_content }) = borsh::from_slice(args)
        else {
            tracing::warn!("failed to decode RegisterStaticRouteCall");
            return;
        };
        let page = Page { site_id: self.site_id, path: route, brotli_html_content };
        self.db.write_page(page);
    }

    fn kv_insert(&mut self, args: &[u8]) {
        let Ok(KeyValueInsertCall { key, value }) = borsh::from_slice(args) else {
            tracing::warn!("failed to decode KeyValueInsert");
            return;
        };
        if value.is_empty() {
            self.db.delete_kv(&key, self.site_id);
        } else {
            self.db.write_kv(&key, value, self.site_id);
        }
    }

    fn kv_get(&self, args: &[u8]) -> Vec<u8> {
        let Ok(KeyValueReadCall { key }) = borsh::from_slice(args) else {
            tracing::warn!("failed to decode KeyValueRead");
            return KeyValueReadResponse { value: vec![] }.encode();
        };
        let value = self.db.read_kv(&key, self.site_id).unwrap_or(vec![]);
        let response = KeyValueReadResponse { value };
        return response.encode();
    }

    fn log(&mut self, args: &[u8]) {
        let Ok(LogCall { message }) = borsh::from_slice(args) else {
            tracing::warn!("failed to decode Log");
            return;
        };
        tracing::info!(site_id = ?self.site_id, "{}", message);
    }
}
use crate::db::BatchDb;
use vastrum_runtime_shared::{
    Ed25519PublicKey, GetMessageSenderResponse, KeyValueInsertCall, KeyValueReadCall,
    KeyValueReadResponse, LogCall, RegisterStaticRouteCall,
};
use vastrum_shared_types::borsh::BorshExt;
use vastrum_shared_types::crypto::{ed25519, sha256::Sha256Digest};
use vastrum_shared_types::types::storage::Page;
use std::sync::Arc;
use vastrum_bindings_host::HostRuntime;
use wasmtime::StoreLimits;
