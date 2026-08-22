#[derive(Clone)]
pub struct IFrameRpcClient {
    site_id: Sha256Digest,
}

impl RpcProvider for IFrameRpcClient {
    type SentTx = IFrameSentTx;

    fn new(site_id: Sha256Digest) -> Self {
        return Self { site_id };
    }

    fn site_id(&self) -> Sha256Digest {
        return self.site_id;
    }

    //iframe does not support setting custom account key currently, web-client handles signing
    fn with_account_key(self, _key: ed25519::PrivateKey) -> Self {
        return self;
    }

    async fn get_key_value(&self, key: String) -> Option<Vec<u8>> {
        let res = vastrum_frontend_lib::get_key_value(key).await?;
        if res.value.is_empty() {
            return None;
        }
        return Some(res.value);
    }

    async fn get_latest_block_height(&self) -> Option<u64> {
        let height = Some(vastrum_frontend_lib::get_latest_block_height().await);
        return height;
    }

    async fn get_tx_hash_inclusion_state(&self, hash: Sha256Digest) -> Result<bool, RpcError> {
        let state = vastrum_frontend_lib::get_tx_hash_inclusion_state(hash).await;
        return Ok(state);
    }

    async fn make_call(&self, calldata: Vec<u8>) -> IFrameSentTx {
        let res = vastrum_frontend_lib::make_call(calldata).await;
        let sent_tx = IFrameSentTx::new(res.tx_hash, res.tx_created_at_block_height);
        return sent_tx;
    }

    async fn make_authenticated_call(&self, calldata: Vec<u8>) -> IFrameSentTx {
        let res = vastrum_frontend_lib::make_authenticated_call(calldata).await;
        let sent_tx = IFrameSentTx::new(res.tx_hash, res.tx_created_at_block_height);
        return sent_tx;
    }
}

pub struct IFrameSentTx {
    tx_hash: Sha256Digest,
    tx_created_at_block_height: u64,
}

impl IFrameSentTx {
    pub fn new(tx_hash: Sha256Digest, tx_created_at_block_height: u64) -> Self {
        Self { tx_hash, tx_created_at_block_height }
    }
}

impl SentTxBehavior for IFrameSentTx {
    fn tx_hash(&self) -> Sha256Digest {
        return self.tx_hash;
    }

    async fn check_if_included(&self) -> bool {
        let state = vastrum_frontend_lib::get_tx_hash_inclusion_state(self.tx_hash).await;
        return state;
    }

    async fn await_confirmation(&self) {
        loop {
            if self.check_if_included().await {
                wait_for_next_block().await;
                return;
            }
            let height = vastrum_frontend_lib::get_latest_block_height().await;
            let pow_expired = height > self.tx_created_at_block_height + VALIDITY_WINDOW;
            if pow_expired {
                return;
            }
            TimeoutFuture::new(100).await;
        }
    }
}

async fn wait_for_next_block() {
    let height = vastrum_frontend_lib::get_latest_block_height().await;
    for _ in 0..240 {
        if vastrum_frontend_lib::get_latest_block_height().await > height {
            return;
        }
        TimeoutFuture::new(500).await;
    }
}

use crate::{RpcError, RpcProvider, SentTxBehavior};
use gloo_timers::future::TimeoutFuture;
use vastrum_shared_types::crypto::{ed25519, sha256::Sha256Digest};
use vastrum_shared_types::limits::VALIDITY_WINDOW;
