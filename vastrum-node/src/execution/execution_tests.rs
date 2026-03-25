fn make_tx(seed: u64, nonce: u64, recent_block_height: u64) -> Transaction {
    let private_key = ed25519::PrivateKey::from_seed(seed);
    let td = TransactionData {
        transaction_type: TransactionType::DeployNewModule,
        calldata: vec![1, 2, 3],
    };
    let compressed = compress_calldata(&td.encode());
    Transaction {
        pub_key: private_key.public_key(),
        signature: private_key.sign_hash(sha256::sha256_hash(&compressed)),
        calldata: compressed,
        nonce: nonce,
        recent_block_height: recent_block_height,
    }
}

fn write_block_with_txs(db: &Arc<Db>, height: u64, txs: Vec<Transaction>) {
    let batch = BatchDb::new(db.clone());
    let block = Block {
        height,
        transactions: txs,
        previous_block_hash: Sha256Digest::from_u64(0),
        timestamp: 0,
        previous_block_state_root: Sha256Digest::default(),
    };
    batch.write_block(FinalizedBlock { block, votes: BTreeMap::new(), round: 0 });
    batch.write_latest_height(height);
    batch.commit();
}

#[test]
fn test_restore_from_disk_rejects_replayed_pow() {
    let db = Arc::new(Db::open_fresh(
        std::env::temp_dir().join("vastrum-test-restore-rejects-replayed-pow"),
    ));
    let tx = make_tx(0xaaa, 1, 1);
    write_block_with_txs(&db, 1, vec![tx.clone()]);

    let execution = Execution::restore_from_disk(db);
    assert!(!execution.verify_pow(&tx), "replayed tx should be rejected");
}

#[test]
fn test_restore_from_disk_allows_unseen_pow() {
    let db = Arc::new(Db::open_fresh(
        std::env::temp_dir().join("vastrum-test-restore-allows-unseen-pow"),
    ));
    let tx_a = make_tx(0xbbb, 1, 1);
    write_block_with_txs(&db, 1, vec![tx_a]);

    let execution = Execution::restore_from_disk(db);
    let tx_b = make_tx(0xbbb, 2, 1);
    assert!(execution.verify_pow(&tx_b), "unseen tx should be accepted");
}

#[test]
fn test_restore_from_disk_prunes_outside_validity_window() {
    let db = Arc::new(Db::open_fresh(
        std::env::temp_dir().join("vastrum-test-restore-prunes-outside-window"),
    ));
    let tx_old = make_tx(0xccc, 1, 1);
    let tx_recent = make_tx(0xddd, 2, 302);

    {
        let batch = BatchDb::new(db.clone());
        let block = Block {
            height: 1,
            transactions: vec![tx_old.clone()],
            previous_block_hash: Sha256Digest::from_u64(0),
            timestamp: 0,
            previous_block_state_root: Sha256Digest::default(),
        };
        batch.write_block(FinalizedBlock { block, votes: BTreeMap::new(), round: 0 });
        batch.commit();
    }
    {
        let batch = BatchDb::new(db.clone());
        for h in 2..=301 {
            let block = Block {
                height: h,
                transactions: vec![],
                previous_block_hash: Sha256Digest::from_u64(0),
                timestamp: 0,
                previous_block_state_root: Sha256Digest::default(),
            };
            batch.write_block(FinalizedBlock { block, votes: BTreeMap::new(), round: 0 });
        }
        batch.commit();
    }
    write_block_with_txs(&db, 302, vec![tx_recent.clone()]);

    let execution = Execution::restore_from_disk(db);

    let pow_hash_old = tx_old.calculate_pow_hash();
    let pow_hash_recent = tx_recent.calculate_pow_hash();

    assert!(
        !execution.seen_pow_hash.contains(&pow_hash_old),
        "pow hash from outside validity window should be pruned"
    );
    assert!(
        execution.seen_pow_hash.contains(&pow_hash_recent),
        "pow hash from inside validity window should be present"
    );
}

#[test]
fn verify_tx() {
    let db = Arc::new(Db::open_fresh(std::env::temp_dir().join("vastrum-test-verify-tx")));
    let private_key = ed25519::PrivateKey::from_seed(0xcadfefe);

    let module_data = vec![123, 33, 12, 55, 123];

    let deploy_website_tx_data = TransactionData {
        transaction_type: TransactionType::DeployNewModule,
        calldata: module_data,
    };
    let compressed = compress_calldata(&deploy_website_tx_data.encode());

    let deploy_website_tx = Transaction {
        pub_key: private_key.public_key(),
        signature: private_key.sign_hash(sha256::sha256_hash(&compressed)),
        calldata: compressed,
        nonce: 0,
        recent_block_height: 0,
    };

    let execution = Execution::new(db);

    assert!(execution.verify_pow(&deploy_website_tx));
}

#[test]
fn verify_call_tx() {
    let db = Arc::new(Db::open_fresh(std::env::temp_dir().join("vastrum-test-verify-call-tx")));
    let private_key = ed25519::PrivateKey::from_seed(0xcadfefe);

    let create_post_json =
        r#"{"signature":"create_post","posttitle":"second post", "postcontent": "hello world"}"#;
    let create_post_tx_data = TransactionData {
        transaction_type: TransactionType::Call,
        calldata: (SiteCall {
            site_id: Sha256Digest::from_u64(0),
            calldata: create_post_json.as_bytes().to_vec(),
        })
        .encode()
        .to_vec(),
    };

    let compressed = compress_calldata(&create_post_tx_data.encode());
    let create_post_tx = Transaction {
        pub_key: private_key.public_key(),
        signature: private_key.sign_hash(sha256::sha256_hash(&compressed)),
        calldata: compressed,
        nonce: 2,
        recent_block_height: 0,
    };

    let execution = Execution::new(db);
    assert!(execution.verify_pow(&create_post_tx));
}

#[test]
fn test_pow_hash_rejected_after_spent() {
    let db = Arc::new(Db::open_fresh(
        std::env::temp_dir().join("vastrum-test-pow-hash-rejected-after-spent"),
    ));
    let tx = make_tx(0xeee, 1, 0);
    let mut execution = Execution::new(db);

    assert!(execution.verify_pow(&tx), "fresh tx should be accepted");
    execution.mark_pow_as_spent(tx.calculate_pow_hash());
    assert!(!execution.verify_pow(&tx), "spent pow hash should be rejected");
}

#[test]
fn test_pow_expired_outside_validity_window() {
    let db = Arc::new(Db::open_fresh(
        std::env::temp_dir().join("vastrum-test-pow-expired-outside-window"),
    ));
    let tx = make_tx(0xeee, 1, 0);
    let mut execution = Execution::new(db);
    execution.current_block_height = 301;

    assert!(!execution.verify_pow(&tx), "recent_block_height=0 should be expired at height 301");
}

#[test]
fn test_pow_at_window_boundary_accepted() {
    let db =
        Arc::new(Db::open_fresh(std::env::temp_dir().join("vastrum-test-pow-at-window-boundary")));
    let tx = make_tx(0xeee, 1, 0);
    let mut execution = Execution::new(db);
    execution.current_block_height = 300;

    assert!(execution.verify_pow(&tx), "recent_block_height=0 should be accepted at height 300");
}

#[test]
fn test_pow_future_height_rejected() {
    let db = Arc::new(Db::open_fresh(
        std::env::temp_dir().join("vastrum-test-pow-future-height-rejected"),
    ));
    let tx = make_tx(0xeee, 1, 6);
    let mut execution = Execution::new(db);
    execution.current_block_height = 5;

    assert!(!execution.verify_pow(&tx), "recent_block_height=6 should be rejected at height 5");
}

#[test]
fn test_pow_replay_rejected_at_boundary() {
    let db =
        Arc::new(Db::open_fresh(std::env::temp_dir().join("vastrum-test-pow-replay-at-boundary")));
    let tx = make_tx(0xfff, 1, 100); // recent_block_height = 100
    let mut execution = Execution::new(db);

    execution.current_block_height = 100;
    execution.mark_pow_as_spent(tx.calculate_pow_hash());

    execution.current_block_height = 400;
    execution.prune_spent_pow_hashes();

    assert!(
        !execution.verify_pow(&tx),
        "spent PoW at exact boundary must still be rejected (no replay)"
    );
}

use crate::{
    consensus::types::{Block, FinalizedBlock},
    db::{BatchDb, Db},
    execution::execution::Execution,
};
use vastrum_shared_types::{
    borsh::BorshExt,
    crypto::{ed25519, sha256, sha256::Sha256Digest},
    transactioning::compression::compress_calldata,
    types::{
        application::{
            sitecall::SiteCall,
            transactiondata::{TransactionData, TransactionType},
        },
        execution::transaction::Transaction,
    },
};
use std::{collections::BTreeMap, sync::Arc};
