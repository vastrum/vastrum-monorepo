use super::*;

//tests that states is updated correctly inside a block, that batchdb does not just route reads to old state
#[tokio::test]
#[serial]
async fn test_batchdb_readback_multiple_transactions_same_block_works() {
    let ctx = TestContext::new().await;

    let mut txs = Vec::new();
    for _ in 0..5 {
        txs.push(ctx.client.kvmap_increment("counter").await);
    }
    for tx in txs {
        tx.await_confirmation().await;
    }

    let state = ctx.client.state().await;
    assert_eq!(state.kvmap.get(&"counter".to_string()).await, Some(5));
}
