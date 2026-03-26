pub async fn deploy_module(module_path: &str, constructor_calldata: Vec<u8>) -> Sha256Digest {
    let (site_id, tx) =
        deploy_module_tx(module_path.to_string(), constructor_calldata).await.unwrap();
    let http = NativeHttpClient::new();
    http.submit_transaction(tx.encode()).await.unwrap();
    return site_id;
}

pub async fn add_module(module_path: &str) -> Sha256Digest {
    let (module_id, tx) = add_module_tx(module_path.to_string()).await.unwrap();
    let http = NativeHttpClient::new();
    http.submit_transaction(tx.encode()).await.unwrap();
    return module_id;
}

pub async fn instantiate_module(
    module_id: Sha256Digest,
    constructor_calldata: Vec<u8>,
) -> Sha256Digest {
    let (site_id, tx) = instantiate_module_tx(module_id, constructor_calldata).await.unwrap();
    let http = NativeHttpClient::new();
    http.submit_transaction(tx.encode()).await.unwrap();
    return site_id;
}

pub async fn register_domain(
    site_id: Sha256Digest,
    domain_name: impl Into<String>,
) -> NativeTxPoller {
    let private_key = ed25519::PrivateKey::from_seed(0xcadfefe);
    let http = NativeHttpClient::new();
    let recent_block_height = http.get_latest_block_height().await.unwrap();

    let tx = build_register_domain_transaction(
        site_id,
        domain_name.into(),
        rand::random(),
        private_key,
        recent_block_height,
    );

    let tx_hash = tx.calculate_txhash();
    http.submit_transaction(tx.encode()).await.unwrap();
    let sent_tx = NativeTxPoller::new(tx_hash);
    return sent_tx;
}

pub async fn deploy_module_tx(
    module_path: String,
    constructor_calldata: Vec<u8>,
) -> Result<(Sha256Digest, Transaction), HttpError> {
    let http = NativeHttpClient::new();
    let private_key = ed25519::PrivateKey::from_seed(0xcadfefe);
    let wasm_data = std::fs::read(module_path).unwrap();
    let recent_block_height = http.get_latest_block_height().await?;

    let tx = build_deploy_new_module_transaction(
        wasm_data,
        constructor_calldata,
        rand::random(),
        private_key,
        recent_block_height,
    );
    let site_id = tx.calculate_txhash();
    return Ok((site_id, tx));
}

async fn add_module_tx(module_path: String) -> Result<(Sha256Digest, Transaction), HttpError> {
    let private_key = ed25519::PrivateKey::from_seed(0xcadfefe);
    let module_data = std::fs::read(module_path).unwrap();
    let module_id = sha256_hash(&module_data);
    let http = NativeHttpClient::new();
    let recent_block_height = http.get_latest_block_height().await?;

    let tx =
        build_add_module_transaction(module_data, rand::random(), private_key, recent_block_height);
    return Ok((module_id, tx));
}

async fn instantiate_module_tx(
    module_id: Sha256Digest,
    constructor_calldata: Vec<u8>,
) -> Result<(Sha256Digest, Transaction), HttpError> {
    let private_key = ed25519::PrivateKey::from_seed(0xcadfefe);
    let http = NativeHttpClient::new();
    let recent_block_height = http.get_latest_block_height().await?;

    let tx = build_deploy_stored_module_transaction(
        module_id,
        constructor_calldata,
        rand::random(),
        private_key,
        recent_block_height,
    );
    let site_id = tx.calculate_txhash();
    return Ok((site_id, tx));
}
pub async fn poll_until_site_id_deployed(site_id: Sha256Digest) {
    let http = NativeHttpClient::new();
    loop {
        if http.get_site_id_is_deployed(site_id).await.unwrap_or(false) {
            break;
        }
        sleep(Duration::from_millis(5)).await;
    }
    //need to wait 1 more block because state proof for this transactions only exists when the block after this is finalized
    http.wait_for_next_block().await;
}

use crate::error::HttpError;
use crate::{NativeHttpClient, NativeTxPoller};
use vastrum_shared_types::{
    borsh::BorshExt,
    crypto::{
        ed25519,
        sha256::{Sha256Digest, sha256_hash},
    },
    transactioning::transaction_generator::{
        build_add_module_transaction, build_deploy_new_module_transaction,
        build_deploy_stored_module_transaction, build_register_domain_transaction,
    },
    types::execution::transaction::Transaction,
};
use std::time::Duration;
use tokio::time::sleep;
