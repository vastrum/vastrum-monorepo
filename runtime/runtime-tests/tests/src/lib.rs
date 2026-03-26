pub use runtime_tests_abi::*;

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    mod auth;
    mod batch_db;
    mod blockchain_indexer;
    mod domain;
    mod kv_delete;
    mod kv_history;
    mod kvbtree;
    mod kvmap;
    mod kvvec;
    mod kvvecbtree;
    mod nested_kv;
    mod page_serving;
    mod rollback;
    mod state_basics;

    use vastrum_shared_types::crypto::ed25519;
    use vastrum_shared_types::crypto::sha256::Sha256Digest;
    use std::time::SystemTime;

    fn ensure_network() {
        vastrum_native_lib::test_support::ensure_localnet("../contract", "../contract/out");
    }

    struct TestContext {
        site_id: Sha256Digest,
        client: ContractAbiClient,
        account_key: ed25519::PrivateKey,
    }

    impl TestContext {
        async fn new() -> Self {
            ensure_network();
            let client =
                ContractAbiClient::deploy("../contract/out/contract.wasm", "init".to_string())
                    .await;
            let site_id = client.site_id();
            let account_key = ed25519::PrivateKey::from_seed(111);
            Self { site_id, client: client.with_account_key(account_key.clone()), account_key }
        }
    }
}
