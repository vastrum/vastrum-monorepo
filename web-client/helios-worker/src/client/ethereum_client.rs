//adapted from helios-ts
//https://github.com/a16z/helios
extern crate console_error_panic_hook;
extern crate web_sys;

pub struct EthereumClient {
    inner: helios_ethereum::EthereumClient,
    chain_id: u64,
    active_subscriptions: HashMap<String, Subscription<Ethereum>>,
}

impl EthereumClient {
    pub fn new(
        execution_rpc: String,
        consensus_rpc: String,
        network: String,
        checkpoint: String,
    ) -> Result<EthereumClient, JsError> {
        console_error_panic_hook::set_once();

        let base = match network.as_str() {
            "mainnet" => networks::mainnet(),
            "sepolia" => networks::sepolia(),
            "holesky" => networks::holesky(),
            other => Err(JsError::new(&format!("invalid network: {other}")))?,
        };

        let chain_id = base.chain.chain_id;

        let checkpoint = Some(
            Some(checkpoint)
                .as_ref()
                .map(|c| c.strip_prefix("0x").unwrap_or(c.as_str()))
                .and_then(|c| B256::from_hex(c).ok())
                .unwrap_or(base.default_checkpoint),
        );

        let consensus_rpc = Url::parse(&consensus_rpc)
            .map_err(|e| JsError::new(&format!("Invalid consensus RPC URL: {e}")))
            .unwrap();

        let execution_rpc = Some(execution_rpc)
            .map(|url| Url::parse(&url))
            .transpose()
            .map_err(|e| JsError::new(&format!("Invalid execution RPC URL: {e}")))?;

        let config = Config {
            execution_rpc,
            consensus_rpc,
            checkpoint,

            chain: base.chain,
            forks: base.forks,

            ..Default::default()
        };

        let inner = map_err(EthereumClientBuilder::<ConfigDB>::new().config(config).build())?;

        Ok(Self { inner, chain_id, active_subscriptions: HashMap::new() })
    }

    pub async fn subscribe(
        &mut self,
        sub_type: JsValue,
        id: String,
        callback: Function,
    ) -> Result<bool, JsError> {
        let sub_type: SubscriptionType = serde_wasm_bindgen::from_value(sub_type)?;
        let rx = map_err(self.inner.subscribe(sub_type).await)?;

        let subscription = Subscription::<Ethereum>::new(id.clone());

        subscription.listen(rx, callback).await;
        self.active_subscriptions.insert(id, subscription);

        Ok(true)
    }

    pub fn unsubscribe(&mut self, id: String) -> Result<bool, JsError> {
        Ok(self.active_subscriptions.remove(&id).is_some())
    }

    pub async fn wait_synced(&self) -> Result<(), JsError> {
        map_err(self.inner.wait_synced().await)
    }

    pub async fn wait_head_synced(&self) -> Result<(), JsError> {
        map_err(self.inner.wait_head_synced().await)
    }

    pub fn chain_id(&self) -> u32 {
        return self.chain_id as u32;
    }

    pub async fn get_block_number(&self) -> Result<u32> {
        let block_number = self.inner.get_block_number().await;
        let as_u32 = block_number.map(|v| v.to());
        return as_u32;
    }

    pub async fn get_balance(&self, addr: Address, block: BlockId) -> Result<U256> {
        self.inner.get_balance(addr, block).await
    }

    pub async fn get_transaction_by_hash(&self, hash: B256) -> Result<Option<Transaction>> {
        let tx = self.inner.get_transaction(hash).await;
        return tx;
    }

    pub async fn get_transaction_by_block_hash_and_index(
        &self,
        hash: B256,
        index: u64,
    ) -> Result<Option<Transaction>> {
        let tx = self.inner.get_transaction_by_block_and_index(hash.into(), index).await;
        return tx;
    }

    pub async fn get_transaction_by_block_number_and_index(
        &self,
        block: BlockNumberOrTag,
        index: u64,
    ) -> Result<Option<Transaction>> {
        let tx = self.inner.get_transaction_by_block_and_index(block.into(), index).await;
        return tx;
    }

    pub async fn get_transaction_count(&self, addr: Address, block: BlockId) -> Result<u32> {
        let res = self.inner.get_nonce(addr, block).await;
        let as_u32 = res.map(|v| v as u32);
        return as_u32;
    }

    pub async fn get_block_transaction_count_by_hash(&self, hash: B256) -> Result<Option<u32>> {
        let count = self.inner.get_block_transaction_count(hash.into()).await?;
        let as_u32 = count.map(|v| v as u32);
        return Ok(as_u32);
    }

    pub async fn get_block_transaction_count_by_number(
        &self,
        block: BlockNumberOrTag,
    ) -> Result<Option<u32>> {
        let count = self.inner.get_block_transaction_count(block.into()).await?;
        let as_u32 = count.map(|v| v as u32);
        return Ok(as_u32);
    }

    pub async fn get_block_by_number(
        &self,
        block: BlockNumberOrTag,
        full_tx: bool,
    ) -> Result<Option<Block>> {
        let block = self.inner.get_block(block.into(), full_tx).await;
        return block;
    }

    pub async fn get_block_by_hash(&self, hash: B256, full_tx: bool) -> Result<Option<Block>> {
        let block = self.inner.get_block(hash.into(), full_tx).await;
        return block;
    }

    pub async fn get_code(&self, addr: Address, block: BlockId) -> Result<Bytes> {
        let code = self.inner.get_code(addr, block).await;
        return code;
    }

    pub async fn get_storage_at(
        &self,
        address: Address,
        slot: U256,
        block: BlockId,
    ) -> Result<FixedBytes<32>> {
        let storage = self.inner.get_storage_at(address, slot, block).await;
        return storage;
    }

    pub async fn get_proof(
        &self,
        address: Address,
        storage_keys: Vec<FixedBytes<32>>,
        block: BlockId,
    ) -> Result<EIP1186AccountProofResponse> {
        let proof = self.inner.get_proof(address, &storage_keys, block).await;
        return proof;
    }

    pub async fn call(
        &self,
        opts: TransactionRequest,
        block: BlockId,
        state_overrides: Option<StateOverride>,
    ) -> Result<Bytes> {
        let res = self.inner.call(&opts, block, state_overrides).await;
        return res;
    }

    pub async fn estimate_gas(
        &self,
        opts: TransactionRequest,
        block: BlockId,
        state_overrides: Option<StateOverride>,
    ) -> Result<u32> {
        let gas = self.inner.estimate_gas(&opts, block, state_overrides).await;
        let as_u32 = gas.map(|v| v as u32);
        return as_u32;
    }

    pub async fn create_access_list(
        &self,
        opts: TransactionRequest,
        block: BlockId,
        state_overrides: Option<StateOverride>,
    ) -> Result<AccessListResult> {
        let access_list_result = self.inner.create_access_list(&opts, block, state_overrides).await;
        return access_list_result;
    }

    pub async fn gas_price(&self) -> Result<Uint<256, 4>> {
        let price = self.inner.get_gas_price().await;
        return price;
    }

    pub async fn max_priority_fee_per_gas(&self) -> Result<Uint<256, 4>> {
        let price = self.inner.get_priority_fee().await;
        return price;
    }

    pub async fn send_raw_transaction(&self, tx: Vec<u8>) -> Result<FixedBytes<32>> {
        let hash = self.inner.send_raw_transaction(&tx).await;
        return hash;
    }

    pub async fn get_transaction_receipt(&self, tx: B256) -> Result<Option<TransactionReceipt>> {
        let receipt = self.inner.get_transaction_receipt(tx).await;
        return receipt;
    }

    pub async fn get_block_receipts(
        &self,
        block: BlockId,
    ) -> Result<Option<Vec<TransactionReceipt>>> {
        let receipts = self.inner.get_block_receipts(block).await;
        return receipts;
    }

    pub async fn get_logs(&self, filter: Filter) -> Result<Vec<Log>> {
        let logs = self.inner.get_logs(&filter).await;
        return logs;
    }

    pub async fn get_filter_logs(&self, filter_id: U256) -> Result<Vec<Log>> {
        let logs = self.inner.get_filter_logs(filter_id).await;
        return logs;
    }

    pub async fn uninstall_filter(&self, filter_id: U256) -> Result<bool> {
        let uninstalled = self.inner.uninstall_filter(filter_id).await;
        return uninstalled;
    }

    pub async fn new_filter(&self, filter: Filter) -> Result<Uint<256, 4>> {
        let filter_id = self.inner.new_filter(&filter).await;
        return filter_id;
    }

    pub async fn new_block_filter(&self) -> Result<Uint<256, 4>> {
        let filter_id = self.inner.new_block_filter().await;
        return filter_id;
    }

    pub async fn client_version(&self) -> String {
        let version = self.inner.get_client_version().await;
        return version;
    }

    pub async fn get_current_checkpoint(&self) -> Result<Option<FixedBytes<32>>> {
        let checkpoint = self.inner.current_checkpoint().await;
        return checkpoint;
    }
}
fn map_err<T>(val: Result<T>) -> Result<T, JsError> {
    val.map_err(|err| JsError::new(&err.to_string()))
}

use super::subscription::Subscription;
use alloy::eips::{BlockId, BlockNumberOrTag};
use alloy::hex::FromHex;
use alloy::primitives::{Address, B256, Bytes, FixedBytes, U256, Uint};
use alloy::rpc::types::{
    AccessListResult, Block, EIP1186AccountProofResponse, Log, Transaction, TransactionReceipt,
};
use alloy::rpc::types::{Filter, TransactionRequest, state::StateOverride};
use eyre::Result;
use helios_common::types::SubscriptionType;
use helios_ethereum::EthereumClientBuilder;
use helios_ethereum::config::{Config, networks};
use helios_ethereum::database::ConfigDB;
use helios_ethereum::spec::Ethereum;
use std::collections::HashMap;
use url::Url;
use wasm_bindgen::prelude::*;
use web_sys::js_sys::Function;
