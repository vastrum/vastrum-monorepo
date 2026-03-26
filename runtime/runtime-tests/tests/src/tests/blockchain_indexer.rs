use super::*;
use vastrum_native_lib::NativeHttpClient;
use vastrum_shared_types::indexer::types::*;
use vastrum_shared_types::indexer::*;

async fn read_indexer_json<T: serde::de::DeserializeOwned>(key: &str) -> Option<T> {
    let client = NativeHttpClient::new();
    let bytes = client.get_key_value(indexed_blockchain_site_id(), key.to_string()).await?;
    serde_json::from_slice(&bytes).ok()
}

#[tokio::test]
#[serial]
async fn test_latest_height() {
    ensure_network();
    let height: u64 = read_indexer_json(LATEST_HEIGHT_KEY).await.unwrap();
    assert!(height > 0, "latest_height should be > 0, got {height}");
}

#[tokio::test]
#[serial]
async fn test_block_summary() {
    ensure_network();
    let block: BlockSummary = read_indexer_json(&block_key(1)).await.expect("block:1 should exist");
    assert_eq!(block.height, 1);
    assert!(!block.block_hash.is_empty());
    assert!(block.timestamp > 0);
}

#[tokio::test]
#[serial]
async fn test_deploy_indexes_site() {
    let ctx = TestContext::new().await;
    let deployed_id = ctx.site_id.to_string();

    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    let detail: SiteDetail = read_indexer_json(&site_detail_key(&deployed_id))
        .await
        .expect("site_detail should exist after deploy");
    assert_eq!(detail.site_id, deployed_id);
    assert_eq!(detail.deploy_tx, deployed_id);
    assert!(detail.block_height > 0);
    assert!(detail.domain.is_none(), "no domain registered yet");

    let site_count: u64 = read_indexer_json(SITE_COUNT_KEY).await.unwrap_or(0);
    assert!(site_count > 0, "site_count should be > 0 after deploy");

    let last_page = (site_count - 1) / PAGE_SIZE;
    let sites: Vec<String> =
        read_indexer_json(&sites_page_key(last_page)).await.unwrap_or_default();
    assert!(
        sites.iter().any(|s| s == &deployed_id),
        "deployed site {deployed_id} should appear in sites_page:{last_page}"
    );
}

#[tokio::test]
#[serial]
async fn test_tx_detail() {
    let ctx = TestContext::new().await;
    ctx.client.auth_record_sender().await.await_confirmation().await;

    let tx_count: u64 = read_indexer_json(TX_COUNT_KEY).await.unwrap_or(0);
    assert!(tx_count > 0, "tx_count should be > 0");

    let last_page = (tx_count - 1) / PAGE_SIZE;
    let hashes: Vec<String> = read_indexer_json(&txs_page_key(last_page)).await.unwrap_or_default();
    assert!(!hashes.is_empty(), "txs_page should have entries");

    let hash = &hashes[0];
    let detail: TxDetail = read_indexer_json(&tx_key(hash)).await.expect("tx detail should exist");
    assert_eq!(detail.tx_hash, *hash);
    assert!(!detail.tx_type.is_empty());
    assert!(detail.block_height > 0);

    // Verify per-site tx tracking
    let site_id = ctx.site_id.to_string();
    let site_detail: SiteDetail =
        read_indexer_json(&site_detail_key(&site_id)).await.expect("site_detail should exist");
    assert!(site_detail.tx_count > 0, "tx_count should be > 0 after auth call");

    let site_tx_hashes: Vec<String> =
        read_indexer_json(&site_txs_key(&site_id, 0)).await.unwrap_or_default();
    assert!(!site_tx_hashes.is_empty(), "site txs page 0 should have entries");
}

#[tokio::test]
#[serial]
async fn test_account_txs() {
    let ctx = TestContext::new().await;
    ctx.client.auth_record_sender().await.await_confirmation().await;

    let pubkey = ctx.account_key.public_key().to_string();
    let count: u64 = read_indexer_json(&account_tx_count_key(&pubkey)).await.unwrap_or(0);
    assert!(count > 0, "account tx_count should be > 0 for {pubkey}");

    let hashes: Vec<String> =
        read_indexer_json(&account_txs_key(&pubkey, 0)).await.unwrap_or_default();
    assert!(!hashes.is_empty(), "account txs page 0 should have entries");
}

#[tokio::test]
#[serial]
async fn test_block_txs() {
    let ctx = TestContext::new().await;
    let tx_poller = ctx.client.auth_record_sender().await;

    let tx_hash = tx_poller.tx_hash().to_string();
    tx_poller.await_confirmation().await;

    let detail: TxDetail = read_indexer_json(&tx_key(&tx_hash)).await.unwrap();
    let height = detail.block_height;
    let txs: Vec<TxSummary> = read_indexer_json(&block_txs_key(height)).await.unwrap();
    let indexed_tx = txs.first().unwrap();
    assert!(indexed_tx.tx_hash == tx_hash);
    assert!(indexed_tx.target_site == Some(ctx.site_id.to_string()));
    assert!(indexed_tx.tx_type == "Call");
    assert!(indexed_tx.sender == Some(ctx.account_key.public_key().to_string()));
}

#[tokio::test]
#[serial]
async fn test_domain_indexing() {
    let ctx = TestContext::new().await;

    let domain_count_before: u64 = read_indexer_json(DOMAIN_COUNT_KEY).await.unwrap_or(0);

    vastrum_native_lib::deployers::deploy::register_domain(ctx.site_id, "testdomain")
        .await
        .await_confirmation()
        .await;

    let domain_count: u64 = read_indexer_json(DOMAIN_COUNT_KEY).await.unwrap_or(0);
    assert!(domain_count > domain_count_before, "domain_count should increment after registration");

    let last_page = (domain_count - 1) / PAGE_SIZE;
    let domains: Vec<DomainInfo> =
        read_indexer_json(&domains_page_key(last_page)).await.unwrap_or_default();
    assert!(
        domains.iter().any(|d| d.domain_name == "testdomain"),
        "testdomain should appear in domains_page"
    );

    let site_id = ctx.site_id.to_string();
    let detail: SiteDetail =
        read_indexer_json(&site_detail_key(&site_id)).await.expect("site_detail should exist");
    assert_eq!(detail.domain.as_deref(), Some("testdomain"));
}
