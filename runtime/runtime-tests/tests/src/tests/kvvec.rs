use super::*;

#[tokio::test]
#[serial]
async fn test_kvvec_push_and_get() {
    let ctx = TestContext::new().await;

    // Empty state
    let state = ctx.client.state().await;
    assert!(state.kvvec.is_empty().await);
    assert_eq!(state.kvvec.length().await, 0);
    assert_eq!(state.kvvec.get(0).await, None);

    // Push and get
    ctx.client.kvvec_push("first").await.await_confirmation().await;
    ctx.client.kvvec_push("second").await.await_confirmation().await;
    ctx.client.kvvec_push("third").await.await_confirmation().await;

    let state = ctx.client.state().await;
    assert_eq!(state.kvvec.length().await, 3);
    assert!(!state.kvvec.is_empty().await);
    assert_eq!(state.kvvec.get(0).await, Some("first".to_string()));
    assert_eq!(state.kvvec.get(1).await, Some("second".to_string()));
    assert_eq!(state.kvvec.get(2).await, Some("third".to_string()));

    // Out of bounds
    assert_eq!(state.kvvec.get(3).await, None);
    assert_eq!(state.kvvec.get(999).await, None);
    assert_eq!(state.kvvec.get(u64::MAX).await, None);
}

#[tokio::test]
#[serial]
async fn test_kvvec_set_existing() {
    let ctx = TestContext::new().await;

    ctx.client.kvvec_push("first").await.await_confirmation().await;
    ctx.client.kvvec_push("second").await.await_confirmation().await;

    ctx.client.kvvec_set(1, "modified").await.await_confirmation().await;
    let state = ctx.client.state().await;
    assert_eq!(state.kvvec.get(1).await, Some("modified".to_string()));
    assert_eq!(state.kvvec.length().await, 2); // length unchanged
}

#[tokio::test]
#[serial]
async fn test_kvvec_edge_values() {
    let ctx = TestContext::new().await;

    // Empty string
    ctx.client.kvvec_push("").await.await_confirmation().await;

    // Large string
    let large = "x".repeat(10000);
    ctx.client.kvvec_push(&large).await.await_confirmation().await;

    let state = ctx.client.state().await;
    assert_eq!(state.kvvec.get(0).await, Some("".to_string()));
    assert_eq!(state.kvvec.get(1).await, Some(large));
}

#[tokio::test]
#[serial]
async fn test_kvvec_sequential_indices() {
    let ctx = TestContext::new().await;

    for i in 0..20 {
        ctx.client.kvvec_push(format!("item_{}", i)).await.await_confirmation().await;
    }

    let state = ctx.client.state().await;
    assert_eq!(state.kvvec.length().await, 20);
    for i in 0..20 {
        assert_eq!(state.kvvec.get(i).await, Some(format!("item_{}", i)));
    }
    assert_eq!(state.kvvec.get(20).await, None);
}

#[tokio::test]
#[serial]
async fn test_kvvec_struct() {
    let ctx = TestContext::new().await;

    // Get on empty vec
    let state = ctx.client.state().await;
    assert!(state.kvvec_struct.get(0).await.is_none());

    ctx.client.kvvec_struct_push("First title", "First content").await.await_confirmation().await;
    ctx.client.kvvec_struct_push("Second title", "Second content").await.await_confirmation().await;

    let state = ctx.client.state().await;
    assert_eq!(state.kvvec_struct.length().await, 2);

    let item0 = state.kvvec_struct.get(0).await.unwrap();
    assert_eq!(item0.title, "First title");
    assert_eq!(item0.content, "First content");

    let item1 = state.kvvec_struct.get(1).await.unwrap();
    assert_eq!(item1.title, "Second title");

    // Nonexistent index
    assert!(state.kvvec_struct.get(2).await.is_none());
    assert!(state.kvvec_struct.get(999).await.is_none());
}
