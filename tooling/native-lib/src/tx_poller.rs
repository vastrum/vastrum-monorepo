pub struct NativeTxPoller {
    tx_hash: Sha256Digest,
}

impl NativeTxPoller {
    pub fn new(tx_hash: Sha256Digest) -> Self {
        Self { tx_hash }
    }

    pub fn tx_hash(&self) -> Sha256Digest {
        self.tx_hash
    }

    pub async fn await_confirmation(&self) {
        let http = NativeHttpClient::new();

        for _ in 0..2400 {
            if http.get_tx_hash_inclusion_state(self.tx_hash).await.unwrap_or(false) {
                //wait one more block so state proof exists for this transactions (state proofs are delayed 1 block)
                http.wait_for_next_block().await;
                return;
            }
            tokio::time::sleep(std::time::Duration::from_millis(5)).await;
        }
    }
}

use crate::NativeHttpClient;
use vastrum_shared_types::crypto::sha256::Sha256Digest;
