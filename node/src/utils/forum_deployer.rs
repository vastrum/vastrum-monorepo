use rand::Rng;
use shared_types::{
    borsh::BorshExt,
    crypto::ed25519,
    types::{
        application::{
            domaindata::DomainData,
            transactiondata::{TransactionData, TransactionType},
        },
        execution::transaction::Transaction,
    },
};

pub fn deploy_forum_transactions() -> Vec<Transaction> {
    let private_key = ed25519::PrivateKey::from_seed(0xcadfefe);

    let wasm_data_path = "../apps/forum/out/component.wasm";
    let component_data = std::fs::read(wasm_data_path).expect("need origin site data");

    let create_component_website_tx_data = TransactionData {
        transaction_type: TransactionType::DeployNewComponent,
        calldata: component_data,
    };
    let deploy_website_tx = Transaction {
        pub_key: private_key.public_key(),
        signature: private_key.sign_hash(create_component_website_tx_data.calculate_hash()),
        calldata: create_component_website_tx_data.encode(),
        pow_nonce: rand::thread_rng().r#gen(),
    };

    let site_id = deploy_website_tx.calculate_txhash();
    println!("siteid is {}", site_id.to_string());

    let domain_data = DomainData { site_id: site_id, domain_name: "zkpunks".to_string() };
    let register_domain_tx_data = TransactionData {
        transaction_type: TransactionType::RegisterDomain,
        calldata: domain_data.encode(),
    };
    let register_domain_tx = Transaction {
        pub_key: private_key.public_key(),
        signature: private_key.sign_hash(register_domain_tx_data.calculate_hash()),
        calldata: register_domain_tx_data.encode(),
        pow_nonce: rand::thread_rng().r#gen(),
    };
    return vec![deploy_website_tx, register_domain_tx];
}
