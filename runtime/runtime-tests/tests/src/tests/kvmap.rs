use super::*;

#[tokio::test]
#[serial]
async fn test_kvmap_set_and_get() {
    let ctx = TestContext::new().await;

    let state = ctx.client.state().await;
    assert_eq!(state.kvmap.get(&"nonexistent".to_string()).await, None);
    assert!(!state.kvmap.contains(&"nonexistent".to_string()).await);

    ctx.client.kvmap_set("alice", 100).await.await_confirmation().await;
    ctx.client.kvmap_set("bob", 50).await.await_confirmation().await;

    let state = ctx.client.state().await;
    assert_eq!(state.kvmap.get(&"alice".to_string()).await, Some(100));
    assert_eq!(state.kvmap.get(&"bob".to_string()).await, Some(50));
    assert!(state.kvmap.contains(&"alice".to_string()).await);
    assert!(!state.kvmap.contains(&"nonexistent".to_string()).await);
}

#[tokio::test]
#[serial]
async fn test_kvmap_overwrite() {
    let ctx = TestContext::new().await;

    let state = ctx.client.state().await;
    assert_eq!(state.kvmap.get(&"alice".to_string()).await, None);

    ctx.client.kvmap_set("alice", 100).await.await_confirmation().await;
    let state = ctx.client.state().await;
    assert_eq!(state.kvmap.get(&"alice".to_string()).await, Some(100));

    ctx.client.kvmap_set("alice", 200).await.await_confirmation().await;

    let state = ctx.client.state().await;
    assert_eq!(state.kvmap.get(&"alice".to_string()).await, Some(200));
}

#[tokio::test]
#[serial]
async fn test_kvmap_remove() {
    let ctx = TestContext::new().await;

    ctx.client.kvmap_set("alice", 100).await.await_confirmation().await;
    ctx.client.kvmap_set("bob", 50).await.await_confirmation().await;

    // Remove
    ctx.client.kvmap_remove("bob").await.await_confirmation().await;
    let state = ctx.client.state().await;
    assert_eq!(state.kvmap.get(&"bob".to_string()).await, None);
    assert!(!state.kvmap.contains(&"bob".to_string()).await);

    // Remove nonexistent (no error, alice unchanged)
    ctx.client.kvmap_remove("nonexistent").await.await_confirmation().await;
    let state = ctx.client.state().await;
    assert_eq!(state.kvmap.get(&"alice".to_string()).await, Some(100));

    // Re-insert after remove
    ctx.client.kvmap_set("bob", 999).await.await_confirmation().await;
    let state = ctx.client.state().await;
    assert_eq!(state.kvmap.get(&"bob".to_string()).await, Some(999));

    // Double remove (no error)
    ctx.client.kvmap_remove("bob").await.await_confirmation().await;
    ctx.client.kvmap_remove("bob").await.await_confirmation().await;
    let state = ctx.client.state().await;
    assert_eq!(state.kvmap.get(&"bob".to_string()).await, None);
}

#[tokio::test]
#[serial]
async fn test_kvmap_edge_values() {
    let ctx = TestContext::new().await;

    ctx.client.kvmap_set("zero", 0).await.await_confirmation().await;
    ctx.client.kvmap_set("", 999).await.await_confirmation().await;
    ctx.client.kvmap_set("max", u64::MAX).await.await_confirmation().await;

    let state = ctx.client.state().await;
    assert_eq!(state.kvmap.get(&"zero".to_string()).await, Some(0));
    assert_eq!(state.kvmap.get(&"".to_string()).await, Some(999));
    assert_eq!(state.kvmap.get(&"max".to_string()).await, Some(u64::MAX));
}

#[tokio::test]
#[serial]
#[ignore]
async fn test_kvmap_bulk_insert() {
    let ctx = TestContext::new().await;

    let mut txs = Vec::new();
    for i in 0..50 {
        txs.push(ctx.client.kvmap_set(format!("bulk_{}", i), i as u64).await);
    }
    for tx in txs {
        tx.await_confirmation().await;
    }

    let state = ctx.client.state().await;
    assert_eq!(state.kvmap.get(&"bulk_0".to_string()).await, Some(0));
    assert_eq!(state.kvmap.get(&"bulk_25".to_string()).await, Some(25));
    assert_eq!(state.kvmap.get(&"bulk_49".to_string()).await, Some(49));
}
