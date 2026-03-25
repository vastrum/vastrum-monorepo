pub trait RpcProvider: Sized {
    type SentTx: SentTxBehavior;

    fn new(site_id: Sha256Digest) -> Self;
    fn site_id(&self) -> Sha256Digest;
    fn with_account_key(self, key: ed25519::PrivateKey) -> Self;

    fn get_key_value(&self, key: String) -> impl Future<Output = Option<Vec<u8>>>;

    fn get_key_value_at_height(
        &self,
        key: String,
        height: u64,
    ) -> impl Future<Output = Option<Vec<u8>>>;

    fn get_latest_block_height(&self) -> impl Future<Output = Option<u64>>;

    fn get_tx_hash_inclusion_state(
        &self,
        hash: Sha256Digest,
    ) -> impl Future<Output = Result<bool, RpcError>>;

    fn make_call(&self, calldata: Vec<u8>) -> impl Future<Output = Self::SentTx>;

    fn make_authenticated_call(&self, calldata: Vec<u8>) -> impl Future<Output = Self::SentTx>;
}

#[derive(Debug, Clone)]
pub struct RpcError(pub String);

pub trait SentTxBehavior {
    fn tx_hash(&self) -> Sha256Digest;
    fn check_if_included(&self) -> impl Future<Output = bool>;
    fn await_confirmation(&self) -> impl Future<Output = ()>;
}

#[cfg(not(target_arch = "wasm32"))]
mod native;
#[cfg(not(target_arch = "wasm32"))]
pub use native::{NativeRpcClient, NativeSentTx};

#[cfg(target_arch = "wasm32")]
mod iframe;
#[cfg(target_arch = "wasm32")]
pub use iframe::{IFrameRpcClient, IFrameSentTx};

#[cfg(not(target_arch = "wasm32"))]
pub type RpcClient = NativeRpcClient;
#[cfg(not(target_arch = "wasm32"))]
pub type SentTx = NativeSentTx;

#[cfg(target_arch = "wasm32")]
pub type RpcClient = IFrameRpcClient;
#[cfg(target_arch = "wasm32")]
pub type SentTx = IFrameSentTx;

use vastrum_shared_types::crypto::{ed25519, sha256::Sha256Digest};
use std::future::Future;
