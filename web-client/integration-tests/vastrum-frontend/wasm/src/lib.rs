pub use web_client_integration_tests_abi::*;

use vastrum_shared_types::crypto::sha256::Sha256Digest;
use wasm_bindgen::prelude::*;

macro_rules! log {
    ($($arg:tt)*) => {
        web_sys::console::log_1(&format!($($arg)*).into())
    };
}

fn client() -> ContractAbiClient {
    ContractAbiClient::new(Sha256Digest::from_u64(0))
}

async fn test_get_pub_key() {
    log!("\n[VASTRUM] Testing get_pub_key...");
    let pk = vastrum_frontend_lib::get_pub_key().await;
    let bytes = pk.to_bytes();
    assert_eq!(bytes.len(), 32, "Public key should be 32 bytes");
    assert!(bytes.iter().any(|&b| b != 0), "Public key should not be all zeros");
    log!("PASS: get_pub_key ({})", hex::encode(bytes));
}

async fn test_get_private_salt() {
    log!("\n[VASTRUM] Testing get_private_salt...");
    let salt1 = vastrum_frontend_lib::get_private_salt("test-ns".into()).await;
    let salt2 = vastrum_frontend_lib::get_private_salt("test-ns".into()).await;
    assert_eq!(salt1, salt2, "Same namespace should produce same salt");

    let salt3 = vastrum_frontend_lib::get_private_salt("different-ns".into()).await;
    assert_ne!(salt1, salt3, "Different namespace should produce different salt");
    log!("PASS: get_private_salt");
}

async fn test_set_get_and_tx_inclusion() {
    log!("\n[VASTRUM] Testing set_data + await_confirmation + get_data...");
    let c = client();

    c.set_data("test-key", "test-value").await.await_confirmation().await;

    let value = c.state().await.data.get(&"test-key".to_string()).await;
    assert_eq!(value.as_deref(), Some("test-value"), "get_data should return set value");
    log!("PASS: set_data + await_confirmation + get_data");
}

async fn test_large_payload() {
    log!("\n[VASTRUM] Testing large payload (~10KB round-trip)...");
    let c = client();
    let large = "x".repeat(10_000);

    c.set_data("large-key", large.clone()).await.await_confirmation().await;

    let value = c.state().await.data.get(&"large-key".to_string()).await;
    assert_eq!(value.as_deref(), Some(large.as_str()), "Large value round-trip mismatch");
    log!("PASS: large payload ({}B)", large.len());
}

async fn test_burst_calls() {
    log!("\n[VASTRUM] Testing burst of 10 sequential calls...");
    let c = client();
    let count = 10;

    for i in 0..count {
        c.set_data(format!("burst-{i}"), format!("value-{i}")).await.await_confirmation().await;
    }

    let state = c.state().await;
    for i in 0..count {
        let val = state.data.get(&format!("burst-{i}")).await;
        assert_eq!(val.as_deref(), Some(format!("value-{i}").as_str()), "burst-{i} mismatch");
    }
    log!("PASS: burst calls ({count} round-trips verified)");
}

#[wasm_bindgen]
pub async fn run_tests() {
    test_get_pub_key().await;
    test_get_private_salt().await;
    test_set_get_and_tx_inclusion().await;
    test_large_payload().await;
    test_burst_calls().await;
}
