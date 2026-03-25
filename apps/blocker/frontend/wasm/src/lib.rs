use vastrum_shared_types::indexer::types::*;
use vastrum_shared_types::indexer::*;
use vastrum_frontend_lib::get_key_value_by_site_id;
use wasm_bindgen::prelude::*;

async fn read_key(key: &str) -> Vec<u8> {
    let res = get_key_value_by_site_id(indexed_blockchain_site_id(), key.to_string()).await;
    res.value
}

#[wasm_bindgen]
pub fn get_page_size() -> u64 {
    PAGE_SIZE
}

#[wasm_bindgen]
pub async fn get_latest_height() -> u64 {
    let data = read_key(LATEST_HEIGHT_KEY).await;
    serde_json::from_slice::<u64>(&data).unwrap_or(0)
}

#[wasm_bindgen]
pub async fn get_block(height: u64) -> JsValue {
    let data = read_key(&block_key(height)).await;
    let block: Option<BlockSummary> = serde_json::from_slice(&data).ok();
    serde_wasm_bindgen::to_value(&block).unwrap()
}

#[wasm_bindgen]
pub async fn get_block_txs(height: u64) -> JsValue {
    let data = read_key(&block_txs_key(height)).await;
    let txs: Vec<TxSummary> = serde_json::from_slice(&data).unwrap_or_default();
    serde_wasm_bindgen::to_value(&txs).unwrap()
}

#[wasm_bindgen]
pub async fn get_tx_detail(hash: String) -> JsValue {
    let data = read_key(&tx_key(&hash)).await;
    let detail: Option<TxDetail> = serde_json::from_slice(&data).ok();
    serde_wasm_bindgen::to_value(&detail).unwrap()
}

#[wasm_bindgen]
pub async fn get_account_tx_count(pubkey: String) -> u64 {
    let data = read_key(&account_tx_count_key(&pubkey)).await;
    serde_json::from_slice(&data).unwrap_or(0)
}

#[wasm_bindgen]
pub async fn get_account_txs(pubkey: String, page: u64) -> Vec<String> {
    let data = read_key(&account_txs_key(&pubkey, page)).await;
    serde_json::from_slice(&data).unwrap_or_default()
}

#[wasm_bindgen]
pub async fn get_tx_count() -> u64 {
    let data = read_key(TX_COUNT_KEY).await;
    serde_json::from_slice(&data).unwrap_or(0)
}

#[wasm_bindgen]
pub async fn get_txs_page(page: u64) -> Vec<String> {
    let data = read_key(&txs_page_key(page)).await;
    serde_json::from_slice(&data).unwrap_or_default()
}

#[wasm_bindgen]
pub async fn get_site_count() -> u64 {
    let data = read_key(SITE_COUNT_KEY).await;
    serde_json::from_slice(&data).unwrap_or(0)
}

#[wasm_bindgen]
pub async fn get_sites_page(page: u64) -> Vec<String> {
    let data = read_key(&sites_page_key(page)).await;
    serde_json::from_slice(&data).unwrap_or_default()
}

#[wasm_bindgen]
pub async fn get_site_detail(site_id: String) -> JsValue {
    let data = read_key(&site_detail_key(&site_id)).await;
    let detail: Option<SiteDetail> = serde_json::from_slice(&data).ok();
    serde_wasm_bindgen::to_value(&detail).unwrap()
}

#[wasm_bindgen]
pub async fn get_site_txs(site_id: String, page: u64) -> Vec<String> {
    let data = read_key(&site_txs_key(&site_id, page)).await;
    serde_json::from_slice(&data).unwrap_or_default()
}

#[wasm_bindgen]
pub async fn get_domain_count() -> u64 {
    let data = read_key(DOMAIN_COUNT_KEY).await;
    serde_json::from_slice(&data).unwrap_or(0)
}

#[wasm_bindgen]
pub async fn get_domains_page(page: u64) -> JsValue {
    let data = read_key(&domains_page_key(page)).await;
    let domains: Vec<DomainInfo> = serde_json::from_slice(&data).unwrap_or_default();
    serde_wasm_bindgen::to_value(&domains).unwrap()
}
