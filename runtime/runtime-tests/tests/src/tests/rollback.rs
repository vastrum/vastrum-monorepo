use super::*;

#[tokio::test]
#[serial]
async fn test_panic_reverts_state() {
    let ctx = TestContext::new().await;

    // Set baseline state across KvMap, scalar, and KvVec
    ctx.client.kvmap_set("alice", 10).await.await_confirmation().await;
    ctx.client.add_to_counter(5).await.await_confirmation().await;
    ctx.client.kvvec_push("baseline").await.await_confirmation().await;

    let state = ctx.client.state().await;
    assert_eq!(state.kvmap.get(&"alice".to_string()).await, Some(10));
    assert_eq!(state.counter, 5);
    assert_eq!(state.kvvec.length().await, 1);

    // Call method that writes state then panics - all writes should be reverted
    ctx.client.write_then_panic("alice", 999).await.await_confirmation().await;

    let state = ctx.client.state().await;
    assert_eq!(state.kvmap.get(&"alice".to_string()).await, Some(10));
    assert_eq!(state.counter, 5);
    assert_eq!(state.kvvec.length().await, 1);
    assert_eq!(state.kvvec.get(0).await, Some("baseline".to_string()));
}

#[tokio::test]
#[serial]
async fn test_contract_works_after_panic() {
    let ctx = TestContext::new().await;

    // Set baseline
    ctx.client.kvmap_set("alice", 10).await.await_confirmation().await;
    ctx.client.add_to_counter(5).await.await_confirmation().await;
    ctx.client.kvvec_push("baseline").await.await_confirmation().await;

    // Panicking call - should revert
    ctx.client.write_then_panic("alice", 999).await.await_confirmation().await;

    // Normal call after panic - contract should still work
    ctx.client.kvmap_set("alice", 20).await.await_confirmation().await;
    ctx.client.add_to_counter(3).await.await_confirmation().await;
    ctx.client.kvvec_push("after panic").await.await_confirmation().await;

    let state = ctx.client.state().await;
    assert_eq!(state.kvmap.get(&"alice".to_string()).await, Some(20));
    assert_eq!(state.counter, 8);
    assert_eq!(state.kvvec.length().await, 2);
    assert_eq!(state.kvvec.get(1).await, Some("after panic".to_string()));
}
