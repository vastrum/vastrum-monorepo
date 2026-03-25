pub mod types;

use crate::crypto::sha256::{Sha256Digest, sha256_hash};

pub const PAGE_SIZE: u64 = 25;

pub fn indexed_blockchain_site_id() -> Sha256Digest {
    sha256_hash(b"indexed-blockchain-data-kv")
}

pub const LATEST_HEIGHT_KEY: &str = "latest_height";
pub const SITE_COUNT_KEY: &str = "site_count";
pub const TX_COUNT_KEY: &str = "tx_count";
pub const DOMAIN_COUNT_KEY: &str = "domain_count";

pub fn block_key(height: u64) -> String {
    format!("block:{height}")
}

pub fn block_txs_key(height: u64) -> String {
    format!("block_txs:{height}")
}

pub fn tx_key(hash: &str) -> String {
    format!("tx:{hash}")
}

pub fn site_detail_key(site_id: &str) -> String {
    format!("site_detail:{site_id}")
}

pub fn sites_page_key(page: u64) -> String {
    format!("sites_page:{page}")
}

pub fn site_txs_key(site_id: &str, page: u64) -> String {
    format!("site:{site_id}:txs:{page}")
}

pub fn txs_page_key(page: u64) -> String {
    format!("txs_page:{page}")
}

pub fn domains_page_key(page: u64) -> String {
    format!("domains_page:{page}")
}

pub fn account_tx_count_key(pubkey: &str) -> String {
    format!("account:{pubkey}:tx_count")
}

pub fn account_txs_key(pubkey: &str, page: u64) -> String {
    format!("account:{pubkey}:txs:{page}")
}
