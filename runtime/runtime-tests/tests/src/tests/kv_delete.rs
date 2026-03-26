use super::*;

#[tokio::test]
#[serial]
async fn test_kv_delete_basic() {
    let ctx = TestContext::new().await;

    //insert key
    ctx.client.kv_insert_raw("mykey", vec![1, 2, 3]).await.await_confirmation().await;
    ctx.client.kv_check_raw_exists("mykey").await.await_confirmation().await;
    let state = ctx.client.state().await;
    assert_eq!(state.message, "exists");

    //delete key
    ctx.client.kv_delete_raw("mykey").await.await_confirmation().await;
    ctx.client.kv_check_raw_exists("mykey").await.await_confirmation().await;
    let state = ctx.client.state().await;
    assert_eq!(state.message, "");
}

#[tokio::test]
#[serial]
async fn test_kv_delete_nonexistent() {
    let ctx = TestContext::new().await;

    ctx.client.kv_delete_raw("nokey").await.await_confirmation().await;
    ctx.client.kv_check_raw_exists("nokey").await.await_confirmation().await;
    let state = ctx.client.state().await;
    assert_eq!(state.message, "");
}

#[tokio::test]
#[serial]
async fn test_kv_insert_after_delete() {
    let ctx = TestContext::new().await;

    //insert, delete, reinsert
    ctx.client.kv_insert_raw("rekey", vec![10, 20]).await.await_confirmation().await;
    ctx.client.kv_delete_raw("rekey").await.await_confirmation().await;
    ctx.client.kv_insert_raw("rekey", vec![30, 40]).await.await_confirmation().await;

    ctx.client.kv_check_raw_exists("rekey").await.await_confirmation().await;
    let state = ctx.client.state().await;
    assert_eq!(state.message, "exists");
}
