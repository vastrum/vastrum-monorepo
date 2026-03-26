/// Index a block so it is accessible from site environment in order to be able build blockchain explorer (blocker)
/// Custom adhoc serialization format
pub fn index_finalized_block(db: &BatchDb, cert: &FinalizedBlock) {
    let height = cert.block.height;
    //height of latest finalized block
    index_latest_finalized_height(db, height);

    //index latest finalized block at block height so blocker can read block data for each block height
    index_latest_finalized_block(db, cert, height);

    //go through all transaction in current block and add it to various indexes
    //for example need to index transactions
    //per sender (account history page)
    //per site_id (contract history page)
    //per block (view contracts in block)
    index_transactions_in_block(db, cert, height);
}

fn index_latest_finalized_height(db: &BatchDb, height: u64) {
    write_json(db, LATEST_HEIGHT_KEY, &height);
}
fn index_latest_finalized_block(db: &BatchDb, cert: &FinalizedBlock, height: u64) {
    let block = &cert.block;
    let summary = BlockSummary {
        height,
        block_hash: block.calculate_hash().to_string(),
        previous_block_hash: block.previous_block_hash.to_string(),
        timestamp: block.timestamp,
        tx_count: block.transactions.len() as u32,
    };
    write_json(db, &block_key(height), &summary);
}
fn index_transactions_in_block(db: &BatchDb, cert: &FinalizedBlock, height: u64) {
    let block = &cert.block;

    let mut tx_summaries = Vec::new();
    for (idx, tx) in block.transactions.iter().enumerate() {
        let tx_hash = tx.calculate_txhash();

        let Ok(decompressed) = decompress_calldata(&tx.calldata) else {
            continue;
        };
        let Ok(tx_data) = borsh::from_slice::<TransactionData>(&decompressed) else {
            continue;
        };

        let (detail, summary) = build_tx_records(
            &tx_data,
            &tx_hash,
            &tx.pub_key,
            height,
            idx,
            block.timestamp,
            tx.nonce,
            tx.recent_block_height,
        );

        // /tx/:hash
        store_tx_detail(db, &detail.tx_hash, &detail);

        // /account/:pubkey
        if let Some(ref s) = detail.sender {
            update_account_tx(db, s, &detail.tx_hash);
        }
        // /transactions
        update_global_tx_list(db, &detail.tx_hash);

        // /sites, /site/:id
        if matches!(
            tx_data.transaction_type,
            TransactionType::DeployNewModule | TransactionType::DeployStoredModule
        ) {
            track_site_deploy(db, &tx_hash, &tx_data, height);
        }

        // /site/:id (domain field)
        if tx_data.transaction_type == TransactionType::RegisterDomain {
            track_domain_register(db, &tx_data.calldata, height);
        }

        // /site/:id (tx history)
        if let Some(ref site) = detail.target_site {
            if tx_data.transaction_type == TransactionType::Call {
                update_site_tx(db, site, &detail.tx_hash);
            }
        }

        tx_summaries.push(summary);
    }
    // /block/:height
    store_block_tx_summaries(db, height, &tx_summaries);
}

fn store_tx_detail(db: &BatchDb, tx_hash: &str, detail: &TxDetail) {
    write_json(db, &tx_key(tx_hash), detail);
}

fn store_block_tx_summaries(db: &BatchDb, height: u64, summaries: &[TxSummary]) {
    write_json(db, &block_txs_key(height), summaries);
}

fn build_tx_records(
    tx_data: &TransactionData,
    tx_hash: &Sha256Digest,
    pub_key: &ed25519::PublicKey,
    block_height: u64,
    tx_index: usize,
    timestamp: u64,
    nonce: u64,
    recent_block_height: u64,
) -> (TxDetail, TxSummary) {
    let (tx_type, target_site, sender, function_sig) = match tx_data.transaction_type {
        TransactionType::Call => {
            let call = borsh::from_slice::<SiteCall>(&tx_data.calldata).ok();
            let sig = call.as_ref().and_then(|c| extract_function_sig(&c.calldata));
            ("Call", call.map(|c| c.site_id.to_string()), Some(pub_key.to_string()), sig)
        }
        TransactionType::DeployNewModule => {
            ("DeployNewModule", Some(tx_hash.to_string()), None, None)
        }
        TransactionType::DeployStoredModule => {
            ("DeployStoredModule", Some(tx_hash.to_string()), None, None)
        }
        TransactionType::RegisterDomain => ("RegisterDomain", None, None, None),
        TransactionType::AddModule => ("AddModule", None, None, None),
    };

    let detail = TxDetail {
        tx_hash: tx_hash.to_string(),
        block_height,
        tx_index: tx_index as u32,
        timestamp,
        sender: sender.clone(),
        tx_type: tx_type.into(),
        target_site: target_site.clone(),
        nonce: nonce.to_string(),
        recent_block_height,
        function_sig: function_sig.clone(),
    };

    let summary = TxSummary {
        tx_hash: tx_hash.to_string(),
        sender,
        tx_type: tx_type.into(),
        target_site,
        function_sig,
    };

    (detail, summary)
}

fn extract_function_sig(calldata: &[u8]) -> Option<String> {
    if calldata.len() < 8 {
        return None;
    }
    Some(format!("0x{}", calldata[..8].iter().map(|b| format!("{b:02x}")).collect::<String>()))
}

fn write_json<T: Serialize + ?Sized>(db: &BatchDb, key: &str, value: &T) {
    db.write_kv(key, serde_json::to_vec(value).unwrap(), indexed_blockchain_site_id());
}

fn read_json<T: DeserializeOwned>(db: &BatchDb, key: &str) -> Option<T> {
    serde_json::from_slice(&db.read_kv(key, indexed_blockchain_site_id())?).ok()
}

fn update_account_tx(db: &BatchDb, sender: &str, tx_hash: &str) {
    let count_key = account_tx_count_key(sender);
    let count: u64 = read_json(db, &count_key).unwrap_or(0);

    let page_key = account_txs_key(sender, count / PAGE_SIZE);
    let mut page_hashes: Vec<String> = read_json(db, &page_key).unwrap_or_default();
    page_hashes.push(tx_hash.to_string());
    write_json(db, &page_key, &page_hashes);

    write_json(db, &count_key, &(count + 1));
}

fn track_site_deploy(db: &BatchDb, tx_hash: &Sha256Digest, tx_data: &TransactionData, height: u64) {
    let site_id = tx_hash.to_string();
    store_site_detail(db, &site_id, tx_hash, tx_data, height);
    append_to_sites_list(db, &site_id);
}

fn store_site_detail(db: &BatchDb, site_id: &str, tx_hash: &Sha256Digest, tx_data: &TransactionData, height: u64) {
    let module_id = match tx_data.transaction_type {
        TransactionType::DeployStoredModule => {
            borsh::from_slice::<DeployStoredModuleCall>(&tx_data.calldata)
                .ok()
                .map(|c| c.module_id.to_string())
        }
        _ => None,
    };

    let detail = SiteDetail {
        site_id: site_id.to_string(),
        module_id,
        deploy_tx: tx_hash.to_string(),
        block_height: height,
        domain: None,
        tx_count: 0,
    };

    write_json(db, &site_detail_key(site_id), &detail);
}

fn append_to_sites_list(db: &BatchDb, site_id: &str) {
    let count: u64 = read_json(db, SITE_COUNT_KEY).unwrap_or(0);
    let page_key = sites_page_key(count / PAGE_SIZE);
    let mut page_sites: Vec<String> = read_json(db, &page_key).unwrap_or_default();
    page_sites.push(site_id.to_string());
    write_json(db, &page_key, &page_sites);
    write_json(db, SITE_COUNT_KEY, &(count + 1));
}

fn track_domain_register(db: &BatchDb, calldata: &[u8], height: u64) {
    let Ok(domain_data) = DomainData::decode(calldata) else {
        return;
    };

    let existing = db.read_domain(&domain_data.domain_name);
    if existing.as_ref().map(|d| &d.site_id) != Some(&domain_data.site_id) {
        return;
    }

    set_site_domain(db, &domain_data);

    let info = DomainInfo {
        domain_name: domain_data.domain_name.clone(),
        site_id: domain_data.site_id.to_string(),
        block_height: height,
    };
    append_to_domains_list(db, info);
}

fn set_site_domain(db: &BatchDb, domain_data: &DomainData) {
    let sk = site_detail_key(&domain_data.site_id.to_string());
    if let Some(mut detail) = read_json::<SiteDetail>(db, &sk) {
        if detail.domain.is_none() {
            detail.domain = Some(domain_data.domain_name.clone());
            write_json(db, &sk, &detail);
        }
    }
}

fn append_to_domains_list(db: &BatchDb, info: DomainInfo) {
    let count: u64 = read_json(db, DOMAIN_COUNT_KEY).unwrap_or(0);
    let page_key = domains_page_key(count / PAGE_SIZE);
    let mut page_domains: Vec<DomainInfo> = read_json(db, &page_key).unwrap_or_default();
    page_domains.push(info);
    write_json(db, &page_key, &page_domains);
    write_json(db, DOMAIN_COUNT_KEY, &(count + 1));
}

fn update_global_tx_list(db: &BatchDb, tx_hash: &str) {
    let count: u64 = read_json(db, TX_COUNT_KEY).unwrap_or(0);
    let page_key = txs_page_key(count / PAGE_SIZE);
    let mut hashes: Vec<String> = read_json(db, &page_key).unwrap_or_default();
    hashes.push(tx_hash.to_string());
    write_json(db, &page_key, &hashes);
    write_json(db, TX_COUNT_KEY, &(count + 1));
}

fn update_site_tx(db: &BatchDb, site_id: &str, tx_hash: &str) {
    let sk = site_detail_key(site_id);
    if let Some(mut detail) = read_json::<SiteDetail>(db, &sk) {
        let page_key = site_txs_key(site_id, detail.tx_count / PAGE_SIZE);
        let mut page_hashes: Vec<String> = read_json(db, &page_key).unwrap_or_default();
        page_hashes.push(tx_hash.to_string());
        write_json(db, &page_key, &page_hashes);

        detail.tx_count += 1;
        write_json(db, &sk, &detail);
    }
}

use crate::consensus::types::FinalizedBlock;
use crate::db::BatchDb;
use serde::{Serialize, de::DeserializeOwned};
use vastrum_shared_types::borsh::BorshExt;
use vastrum_shared_types::crypto::ed25519;
use vastrum_shared_types::crypto::sha256::Sha256Digest;
use vastrum_shared_types::indexer::types::*;
use vastrum_shared_types::indexer::*;
use vastrum_shared_types::transactioning::compression::decompress_calldata;
use vastrum_shared_types::types::application::deploy_stored_module::DeployStoredModuleCall;
use vastrum_shared_types::types::application::domaindata::DomainData;
use vastrum_shared_types::types::application::sitecall::SiteCall;
use vastrum_shared_types::types::application::transactiondata::{TransactionData, TransactionType};
