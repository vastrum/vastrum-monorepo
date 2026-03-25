use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct BlockSummary {
    pub height: u64,
    pub block_hash: String,
    pub previous_block_hash: String,
    pub timestamp: u64,
    pub tx_count: u32,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct TxSummary {
    pub tx_hash: String,
    pub sender: Option<String>,
    pub tx_type: String,
    pub target_site: Option<String>,
    pub function_sig: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct TxDetail {
    pub tx_hash: String,
    pub block_height: u64,
    pub tx_index: u32,
    pub timestamp: u64,
    pub sender: Option<String>,
    pub tx_type: String,
    pub target_site: Option<String>,
    pub nonce: String,
    pub recent_block_height: u64,
    pub function_sig: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SiteDetail {
    pub site_id: String,
    pub module_id: Option<String>,
    pub deploy_tx: String,
    pub block_height: u64,
    pub domain: Option<String>,
    pub tx_count: u64,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct DomainInfo {
    pub domain_name: String,
    pub site_id: String,
    pub block_height: u64,
}
