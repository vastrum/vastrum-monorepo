extern crate console_error_panic_hook;
const SYNC_TIMEOUT_MS: u32 = 70_000;

pub async fn resolve_eth_rpc_request(req: EthRPCRequest) -> Result<Value> {
    console_error_panic_hook::set_once();

    let client = ensure_synced().await?;

    match req.method.as_str() {
        "eth_chainId" => {
            let chain_id = client.chain_id();
            let value = serde_json::to_value(format!("0x{:x}", chain_id))?;
            return Ok(value);
        }
        "net_version" => {
            let chain_id = client.chain_id();
            let value = serde_json::to_value(chain_id)?;
            return Ok(value);
        }

        "eth_blockNumber" => {
            let block_number = client.get_block_number().await?;
            let value = serde_json::to_value(format!("0x{:x}", block_number))?;
            return Ok(value);
        }

        "eth_getBalance" => {
            let addr: Address = serde_json::from_value(req.params[0].clone())?;
            let block: BlockId = serde_json::from_value(req.params[1].clone())?;
            let balance = client.get_balance(addr, block).await?;
            let value = serde_json::to_value(format!("0x{:x}", balance))?;
            return Ok(value);
        }

        "eth_getTransactionByHash" => {
            let hash: String = serde_json::from_value(req.params[0].clone())?;
            let hash = B256::from_str(&hash)?;
            let tx = client.get_transaction_by_hash(hash).await?;
            let value = serde_json::to_value(&tx)?;
            return Ok(value);
        }

        "eth_getTransactionByBlockHashAndIndex" => {
            let hash: B256 = serde_json::from_value(req.params[0].clone())?;
            let index: alloy::rpc::types::Index = serde_json::from_value(req.params[1].clone())?;
            let tx = client.get_transaction_by_block_hash_and_index(hash, index.0 as u64).await?;
            let value = serde_json::to_value(&tx)?;
            return Ok(value);
        }

        "eth_getTransactionByBlockNumberAndIndex" => {
            let block: BlockNumberOrTag = serde_json::from_value(req.params[0].clone())?;
            let index: alloy::rpc::types::Index = serde_json::from_value(req.params[1].clone())?;
            let tx =
                client.get_transaction_by_block_number_and_index(block, index.0 as u64).await?;
            let value = serde_json::to_value(&tx)?;
            return Ok(value);
        }

        "eth_getTransactionCount" => {
            let addr: Address = serde_json::from_value(req.params[0].clone())?;
            let block: BlockId = serde_json::from_value(req.params[1].clone())?;
            let count = client.get_transaction_count(addr, block).await?;
            let value = serde_json::to_value(format!("0x{:x}", count))?;
            return Ok(value);
        }

        "eth_getBlockTransactionCountByHash" => {
            let hash: B256 = serde_json::from_value(req.params[0].clone())?;
            let count = client.get_block_transaction_count_by_hash(hash).await?;

            let value = match count {
                Some(c) => serde_json::to_value(format!("0x{:x}", c))?,
                None => serde_json::Value::Null,
            };
            return Ok(value);
        }

        "eth_getBlockTransactionCountByNumber" => {
            let block: BlockNumberOrTag = serde_json::from_value(req.params[0].clone())?;
            let count = client.get_block_transaction_count_by_number(block).await?;
            let value = match count {
                Some(c) => serde_json::to_value(format!("0x{:x}", c))?,
                None => serde_json::Value::Null,
            };
            return Ok(value);
        }

        "eth_getBlockByNumber" => {
            let block: BlockNumberOrTag = serde_json::from_value(req.params[0].clone())?;
            let full_tx: bool = serde_json::from_value(req.params[1].clone()).unwrap_or(false);
            let block_data = client.get_block_by_number(block, full_tx).await?;
            let value = serde_json::to_value(&block_data)?;
            return Ok(value);
        }

        "eth_getBlockByHash" => {
            let hash: B256 = serde_json::from_value(req.params[0].clone())?;
            let full_tx: bool = serde_json::from_value(req.params[1].clone()).unwrap_or(false);
            let block = client.get_block_by_hash(hash, full_tx).await?;
            let value = serde_json::to_value(&block)?;
            return Ok(value);
        }

        "eth_getCode" => {
            let addr: Address = serde_json::from_value(req.params[0].clone())?;
            let block: BlockId = serde_json::from_value(req.params[1].clone())?;
            let code = client.get_code(addr, block).await?;
            let value = serde_json::to_value(format!("0x{}", hex::encode(code)))?;
            return Ok(value);
        }

        "eth_getStorageAt" => {
            let addr: Address = serde_json::from_value(req.params[0].clone())?;
            let slot: U256 = serde_json::from_value(req.params[1].clone())?;
            let block: BlockId = serde_json::from_value(req.params[2].clone())?;
            let storage = client.get_storage_at(addr, slot, block).await?;
            let value = serde_json::to_value(storage)?;
            return Ok(value);
        }

        "eth_getProof" => {
            let addr: Address = serde_json::from_value(req.params[0].clone())?;

            let storage_keys: Vec<U256> = serde_json::from_value(req.params[1].clone())?;
            let storage_keys = storage_keys.into_iter().map(|k| k.into()).collect::<Vec<_>>();

            let block: BlockId = serde_json::from_value(req.params[2].clone())?;

            let proof = client.get_proof(addr, storage_keys, block).await?;
            let value = serde_json::to_value(&proof)?;
            return Ok(value);
        }

        "eth_call" => {
            let opts: TransactionRequest = serde_json::from_value(req.params[0].clone())?;
            let block: BlockId = serde_json::from_value(req.params[1].clone())?;
            let state_overrides: Option<StateOverride> =
                req.params.get(2).map(|v| serde_json::from_value(v.clone())).transpose()?;
            let result = client.call(opts, block, state_overrides).await?;
            let value = serde_json::to_value(format!("0x{}", hex::encode(result)))?;

            return Ok(value);
        }

        "eth_estimateGas" => {
            let opts: TransactionRequest = serde_json::from_value(req.params[0].clone())?;
            let block: BlockId = req
                .params
                .get(1)
                .map(|v| serde_json::from_value(v.clone()))
                .transpose()?
                .unwrap_or(BlockId::latest());
            let state_overrides: Option<StateOverride> =
                req.params.get(2).map(|v| serde_json::from_value(v.clone())).transpose()?;
            let gas = client.estimate_gas(opts, block, state_overrides).await?;
            let value = serde_json::to_value(format!("0x{:x}", gas))?;
            return Ok(value);
        }

        "eth_createAccessList" => {
            let opts: TransactionRequest = serde_json::from_value(req.params[0].clone())?;
            let block: BlockId = req
                .params
                .get(1)
                .map(|v| serde_json::from_value(v.clone()))
                .transpose()?
                .unwrap_or(BlockId::latest());
            let state_overrides: Option<StateOverride> =
                req.params.get(2).map(|v| serde_json::from_value(v.clone())).transpose()?;
            let access_list = client.create_access_list(opts, block, state_overrides).await?;
            let value = serde_json::to_value(&access_list)?;
            return Ok(value);
        }

        "eth_gasPrice" => {
            let price = client.gas_price().await?;
            let value = serde_json::to_value(format!("0x{:x}", price))?;
            return Ok(value);
        }

        "eth_maxPriorityFeePerGas" => {
            let price = client.max_priority_fee_per_gas().await?;
            let value = serde_json::to_value(format!("0x{:x}", price))?;
            return Ok(value);
        }

        "eth_sendRawTransaction" => {
            let tx_hex: String = serde_json::from_value(req.params[0].clone())?;
            let tx_bytes = hex::decode(tx_hex)?;
            let hash = client.send_raw_transaction(tx_bytes).await?;
            let value = serde_json::to_value(hash)?;
            return Ok(value);
        }

        "eth_getTransactionReceipt" => {
            let hash: B256 = serde_json::from_value(req.params[0].clone())?;
            let receipt = client.get_transaction_receipt(hash).await?;
            let value = serde_json::to_value(&receipt)?;
            return Ok(value);
        }

        "eth_getBlockReceipts" => {
            let block: BlockId = serde_json::from_value(req.params[0].clone())?;
            let receipts = client.get_block_receipts(block).await?;
            let value = serde_json::to_value(&receipts)?;
            return Ok(value);
        }

        "eth_getLogs" => {
            let filter: Filter = serde_json::from_value(req.params[0].clone())?;
            let logs = client.get_logs(filter).await?;
            let value = serde_json::to_value(&logs)?;
            return Ok(value);
        }

        "eth_getFilterLogs" => {
            let filter_id: U256 = serde_json::from_value(req.params[0].clone())?;
            let logs = client.get_filter_logs(filter_id).await?;
            let value = serde_json::to_value(&logs)?;
            return Ok(value);
        }

        "eth_uninstallFilter" => {
            let filter_id: U256 = serde_json::from_value(req.params[0].clone())?;
            let result = client.uninstall_filter(filter_id).await?;
            let value = serde_json::to_value(result)?;
            return Ok(value);
        }

        "eth_newFilter" => {
            let filter: Filter = serde_json::from_value(req.params[0].clone())?;
            let filter_id = client.new_filter(filter).await?;
            let value = serde_json::to_value(filter_id)?;
            return Ok(value);
        }

        "eth_newBlockFilter" => {
            let filter_id = client.new_block_filter().await?;
            let value = serde_json::to_value(filter_id)?;
            return Ok(value);
        }

        "web3_clientVersion" => {
            let version = client.client_version().await;
            let value = serde_json::to_value(&version)?;
            return Ok(value);
        }
        /*
               "eth_subscribe" | "eth_unsubscribe" => {
                   // TODO: WebSocket support
               }
        */
        _ => Err(eyre!("method not supported: {}", req.method)),
    }
}

async fn ensure_synced() -> Result<Rc<EthereumClient>> {
    let client = get_eth_client();
    match try_sync(&client).await {
        Ok(()) => Ok(client),
        Err(e) => {
            web_sys::console::warn_1(
                &format!("Helios sync failed ({e}), retrying with fresh client").into(),
            );
            reset_eth_client();
            let client = get_eth_client();
            try_sync(&client).await?;
            Ok(client)
        }
    }
}
async fn try_sync(client: &EthereumClient) -> Result<()> {
    match select(Box::pin(client.wait_head_synced()), TimeoutFuture::new(SYNC_TIMEOUT_MS)).await {
        Either::Left((Ok(()), _)) => Ok(()),
        Either::Left((Err(e), _)) => Err(eyre!("Helios sync error: {e:?}")),
        Either::Right(_) => Err(eyre!("Helios sync timed out")),
    }
}

use super::ethereum_client::EthereumClient;
use super::provider::{get_eth_client, reset_eth_client};
use alloy::eips::{BlockId, BlockNumberOrTag};
use alloy::primitives::{Address, B256, U256};
use alloy::rpc::types::{Filter, TransactionRequest, state::StateOverride};
use eyre::{Result, eyre};
use futures::future::{Either, select};
use gloo_timers::future::TimeoutFuture;
use serde_json::Value;
use vastrum_shared_types::iframerpc::types::EthRPCRequest;
use std::rc::Rc;
use std::str::FromStr;
