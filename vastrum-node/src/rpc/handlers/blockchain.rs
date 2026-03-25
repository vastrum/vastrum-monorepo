pub fn get_page(db: &Db, payload: GetPagePayload) -> GetPageResult {
    // First check if site_identifiers is registed as domain
    // Then check if site_identifier can be parsed to sha256digest
    // Otherwise not valid page request
    let site_id = if let Some(domain) = db.read_domain(&payload.site_identifier) {
        domain.site_id
    } else if let Some(site_id) = Sha256Digest::from_string(&payload.site_identifier) {
        site_id
    } else {
        return GetPageResult::Err(ProvedReadError::SiteNotFound);
    };

    let Some((actual_path, brotli_html_content)) = resolve_route(db, site_id, &payload.page_path)
    else {
        return GetPageResult::Err(ProvedReadError::PageNotFound);
    };

    let proof_height = db.read_latest_finalized_height();

    let storage_key = PageStorageKey::new(site_id, &actual_path).encode();
    //state hash is delayed 1 block,
    let state_proof_height = proof_height.saturating_sub(1);
    match db.generate_state_proof("page", &storage_key, state_proof_height) {
        Some(state_proof) => GetPageResult::Ok(PageResponse {
            brotli_html_content,
            site_id,
            page_path: actual_path,
            state_proof,
        }),
        None => GetPageResult::Err(ProvedReadError::ProofUnavailable),
    }
}

pub fn get_latest_block_height(db: &Db) -> GetLatestBlockHeightResponse {
    GetLatestBlockHeightResponse { height: db.read_latest_finalized_height() }
}

pub fn get_key_value(db: &Db, payload: GetKeyValuePayload) -> GetKeyValueResult {
    let current_height = db.read_latest_finalized_height();
    //can only prove current_height -1, if request is above this, then clamp it down to latest provable height
    let height = match payload.height_lock {
        Some(h) if h < current_height => h,
        _ => current_height.saturating_sub(1),
    };

    let does_not_have_height_in_db = height + KV_RETENTION_WINDOW < current_height;
    if does_not_have_height_in_db {
        return GetKeyValueResult::Err(ProvedReadError::OutsideRetentionWindow);
    }

    match db.read_kv_with_proof(&payload.key, payload.site_id, height) {
        Some((value, state_proof)) => {
            GetKeyValueResult::Ok(GetKeyValueResponse { value, state_proof })
        }
        None => GetKeyValueResult::Err(ProvedReadError::ProofUnavailable),
    }
}

pub fn submit(networking: &Networking, payload: SubmitTransactionPayload) {
    if let Ok(transaction) = Transaction::decode(&payload.transaction_bytes) {
        networking.broadcast_transaction(transaction);
    }
}

pub fn get_site_id_is_deployed(
    db: &Db,
    payload: GetSiteIDIsDeployed,
) -> GetSiteIDIsDeployedResponse {
    let result = db.read_site(payload.site_id).is_some();
    GetSiteIDIsDeployedResponse { result }
}

pub fn get_tx_hash_inclusion_state(
    db: &Db,
    payload: GetTxHashIsIncluded,
) -> GetTxHashIsIncludedResponse {
    let included = db.check_tx_inclusion_state(payload.tx_hash);
    GetTxHashIsIncludedResponse { included }
}

pub fn resolve_domain(db: &Db, payload: ResolveDomainRequest) -> ResolveDomainResponse {
    let domain = db.read_domain(&payload.domain);
    let site_id = domain.map(|d| d.site_id);
    ResolveDomainResponse { site_id }
}

fn resolve_route(db: &Db, site_id: Sha256Digest, path: &str) -> Option<(String, Vec<u8>)> {
    //path has registed route for path
    if let Some(page) = db.read_page(site_id, path) {
        return Some((path.to_string(), page.brotli_html_content));
    }

    //otherwise try fallback
    if !path.is_empty() {
        let content = match db.read_page(site_id, "") {
            Some(page) => page.brotli_html_content,
            None => Vec::new(),
        };
        return Some(("".to_string(), content));
    } else {
        return None;
    }
}

use crate::{db::Db, p2p::networking::Networking};
use vastrum_shared_types::borsh::BorshExt;
use vastrum_shared_types::limits::KV_RETENTION_WINDOW;
use vastrum_shared_types::types::storage::PageStorageKey;
use vastrum_shared_types::{
    crypto::sha256::Sha256Digest,
    types::{
        execution::transaction::Transaction,
        rpc::types::{
            GetKeyValuePayload, GetKeyValueResponse, GetKeyValueResult,
            GetLatestBlockHeightResponse, GetPagePayload, GetPageResult, GetSiteIDIsDeployed,
            GetSiteIDIsDeployedResponse, GetTxHashIsIncluded, GetTxHashIsIncludedResponse,
            PageResponse, ProvedReadError, ResolveDomainRequest, ResolveDomainResponse,
            SubmitTransactionPayload,
        },
    },
};
