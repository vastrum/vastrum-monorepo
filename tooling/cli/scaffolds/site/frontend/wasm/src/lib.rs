use {{name_underscore}}_abi::*;
use vastrum_shared_types::crypto::sha256::Sha256Digest;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub async fn set_value(key: String, value: String) -> String {
    let client = ContractAbiClient::new(Sha256Digest::from_u64(0));
    let sent_tx = client.set_value(key, value).await;
    sent_tx.tx_hash().to_string()
}

#[wasm_bindgen]
pub async fn get_value(key: String) -> Option<String> {
    let client = ContractAbiClient::new(Sha256Digest::from_u64(0));
    let state = client.state().await;
    state.store.get(&key).await
}
