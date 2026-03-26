fn build_and_validate_transaction(
    tx_data: &TransactionData,
    private_key: &ed25519::PrivateKey,
    nonce: u64,
    recent_block_height: u64,
) -> Transaction {
    let compressed = compress_calldata(&tx_data.encode());
    let tx = Transaction {
        pub_key: private_key.public_key(),
        signature: private_key.sign_hash(sha256::sha256_hash(&compressed)),
        calldata: compressed,
        nonce,
        recent_block_height,
    };
    let tx_bytes = tx.encode();
    assert!(
        tx_bytes.len() <= MAX_TRANSACTION_SIZE,
        "transaction too large: {} bytes (max {}).",
        tx_bytes.len(),
        MAX_TRANSACTION_SIZE,
    );
    return tx;
}

pub fn build_call_transaction(
    site_id: Sha256Digest,
    calldata: Vec<u8>,
    nonce: u64,
    private_key: ed25519::PrivateKey,
    recent_block_height: u64,
) -> Transaction {
    let tx_data = TransactionData {
        transaction_type: TransactionType::Call,
        calldata: SiteCall { site_id, calldata }.encode(),
    };
    let transaction =
        build_and_validate_transaction(&tx_data, &private_key, nonce, recent_block_height);
    return transaction;
}

pub fn wrap_transaction(transaction: Transaction) -> SubmitTransactionPayload {
    return SubmitTransactionPayload { transaction_bytes: transaction.encode() };
}

fn assert_wasm_module_size(data: &[u8]) {
    assert!(
        data.len() <= MAX_WASM_MODULE_SIZE,
        "compiled contract wasm module too large: {} bytes (max {})",
        data.len(),
        MAX_WASM_MODULE_SIZE,
    );
}

pub fn build_deploy_new_module_transaction(
    wasm_data: Vec<u8>,
    constructor_calldata: Vec<u8>,
    nonce: u64,
    private_key: ed25519::PrivateKey,
    recent_block_height: u64,
) -> Transaction {
    assert_wasm_module_size(&wasm_data);
    let deploy_call = DeployNewModuleCall { wasm_data, constructor_calldata };
    let tx_data = TransactionData {
        transaction_type: TransactionType::DeployNewModule,
        calldata: deploy_call.encode(),
    };
    let tx = build_and_validate_transaction(&tx_data, &private_key, nonce, recent_block_height);
    return tx;
}

pub fn build_add_module_transaction(
    module_data: Vec<u8>,
    nonce: u64,
    private_key: ed25519::PrivateKey,
    recent_block_height: u64,
) -> Transaction {
    assert_wasm_module_size(&module_data);
    let tx_data =
        TransactionData { transaction_type: TransactionType::AddModule, calldata: module_data };
    let tx = build_and_validate_transaction(&tx_data, &private_key, nonce, recent_block_height);
    return tx;
}

pub fn build_deploy_stored_module_transaction(
    module_id: Sha256Digest,
    constructor_calldata: Vec<u8>,
    nonce: u64,
    private_key: ed25519::PrivateKey,
    recent_block_height: u64,
) -> Transaction {
    let deploy_call = DeployStoredModuleCall { module_id, constructor_calldata };
    let tx_data = TransactionData {
        transaction_type: TransactionType::DeployStoredModule,
        calldata: deploy_call.encode(),
    };
    let tx = build_and_validate_transaction(&tx_data, &private_key, nonce, recent_block_height);
    return tx;
}

pub fn build_register_domain_transaction(
    site_id: Sha256Digest,
    domain_name: String,
    nonce: u64,
    private_key: ed25519::PrivateKey,
    recent_block_height: u64,
) -> Transaction {
    let domain_data = DomainData { site_id, domain_name };
    let tx_data = TransactionData {
        transaction_type: TransactionType::RegisterDomain,
        calldata: domain_data.encode(),
    };
    build_and_validate_transaction(&tx_data, &private_key, nonce, recent_block_height)
}

use crate::{
    borsh::BorshExt,
    crypto::{ed25519, sha256, sha256::Sha256Digest},
    limits::{MAX_TRANSACTION_SIZE, MAX_WASM_MODULE_SIZE},
    transactioning::compression::compress_calldata,
    types::{
        application::{
            deploy_new_module::DeployNewModuleCall,
            deploy_stored_module::DeployStoredModuleCall,
            domaindata::DomainData,
            sitecall::SiteCall,
            transactiondata::{TransactionData, TransactionType},
        },
        execution::transaction::Transaction,
        rpc::types::SubmitTransactionPayload,
    },
};
