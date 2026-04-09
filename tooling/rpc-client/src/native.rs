#[derive(Clone)]
pub struct NativeRpcClient {
    site_id: Sha256Digest,
    http: NativeHttpClient,
    account_key: Option<ed25519::PrivateKey>,
}

impl RpcProvider for NativeRpcClient {
    type SentTx = NativeSentTx;

    fn new(site_id: Sha256Digest) -> Self {
        Self { site_id, http: NativeHttpClient::new(), account_key: None }
    }

    fn site_id(&self) -> Sha256Digest {
        return self.site_id;
    }

    fn with_account_key(mut self, key: ed25519::PrivateKey) -> Self {
        self.account_key = Some(key);
        return self;
    }

    async fn get_key_value(&self, key: String) -> Option<Vec<u8>> {
        let result =
            self.http.get_key_value_response(self.site_id, key.clone(), None).await.ok()?;
        let response = match result {
            GetKeyValueResult::Ok(r) => r,
            GetKeyValueResult::Err(e) => {
                eprintln!("get_key_value failed for key {key}: {e:?}");
                return None;
            }
        };
        let genesis = genesis_epoch_state();
        let now = SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
        if let Err(e) = verify_keyvalue_proof(
            &response,
            self.site_id,
            &key,
            &genesis.validators,
            genesis.total_stake,
            now,
        ) {
            eprintln!("proof verification failed for key {key}: {e}");
            return None;
        }
        if response.value.is_empty() {
            return None;
        }
        return Some(response.value);
    }

    async fn get_key_value_at_height(&self, key: String, height: u64) -> Option<Vec<u8>> {
        let result =
            self.http.get_key_value_response(self.site_id, key.clone(), Some(height)).await.ok()?;
        let response = match result {
            GetKeyValueResult::Ok(r) => r,
            GetKeyValueResult::Err(e) => {
                eprintln!("get_key_value failed for key {key} at height {height}: {e:?}");
                return None;
            }
        };
        let genesis = genesis_epoch_state();
        let now = SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
        if let Err(e) = verify_keyvalue_proof(
            &response,
            self.site_id,
            &key,
            &genesis.validators,
            genesis.total_stake,
            now,
        ) {
            eprintln!("proof verification failed for key {key} at height {height}: {e}");
            return None;
        }
        if response.value.is_empty() {
            return None;
        }
        return Some(response.value);
    }

    async fn get_latest_block_height(&self) -> Option<u64> {
        let height = self.http.get_latest_block_height().await.ok();
        return height;
    }

    async fn get_tx_hash_inclusion_state(&self, hash: Sha256Digest) -> Result<bool, RpcError> {
        let state = Ok(self.http.get_tx_hash_inclusion_state(hash).await?);
        return state;
    }

    async fn make_call(&self, calldata: Vec<u8>) -> NativeSentTx {
        let throwaway_private_key = ed25519::PrivateKey::from_rng();
        let nonce = rand::random();

        let recent_block_height = self.http.get_latest_block_height().await.unwrap();

        let transaction = build_call_transaction(
            self.site_id,
            calldata,
            nonce,
            throwaway_private_key,
            recent_block_height,
        );
        self.http.submit_transaction(transaction.encode()).await.unwrap();

        let sent_tx = NativeSentTx::new(transaction.calculate_txhash(), self.http.clone());
        return sent_tx;
    }

    async fn make_authenticated_call(&self, calldata: Vec<u8>) -> NativeSentTx {
        let account_private_key = self
            .account_key
            .clone()
            .expect("Authenticated call requires key. Use .with_account_key()");

        let recent_block_height = self.http.get_latest_block_height().await.unwrap();
        let nonce = rand::random();

        let transaction = build_call_transaction(
            self.site_id,
            calldata,
            nonce,
            account_private_key,
            recent_block_height,
        );
        self.http.submit_transaction(transaction.encode()).await.unwrap();

        let sent_tx = NativeSentTx::new(transaction.calculate_txhash(), self.http.clone());
        return sent_tx;
    }
}

pub struct NativeSentTx {
    tx_hash: Sha256Digest,
    http: NativeHttpClient,
}

impl NativeSentTx {
    pub fn new(tx_hash: Sha256Digest, http: NativeHttpClient) -> Self {
        Self { tx_hash, http }
    }
}

impl SentTxBehavior for NativeSentTx {
    fn tx_hash(&self) -> Sha256Digest {
        self.tx_hash
    }

    async fn check_if_included(&self) -> bool {
        self.http.get_tx_hash_inclusion_state(self.tx_hash).await.unwrap_or(false)
    }

    async fn await_confirmation(&self) {
        NativeTxPoller::new(self.tx_hash).await_confirmation().await;
    }
}

use std::time::SystemTime;

use crate::{RpcError, RpcProvider, SentTxBehavior};
use vastrum_native_lib::{NativeHttpClient, NativeTxPoller};

impl From<vastrum_native_lib::error::HttpError> for RpcError {
    fn from(e: vastrum_native_lib::error::HttpError) -> Self {
        RpcError(e.0)
    }
}
use vastrum_shared_types::{
    borsh::BorshExt,
    crypto::{ed25519, sha256::Sha256Digest},
    genesis::genesis_epoch_state,
    proof_verification::verify_keyvalue_proof,
    transactioning::transaction_generator::build_call_transaction,
    types::rpc::types::GetKeyValueResult,
};
