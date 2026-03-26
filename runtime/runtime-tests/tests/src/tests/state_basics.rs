use super::*;

#[tokio::test]
#[serial]
async fn test_counter() {
    let ctx = TestContext::new().await;
    let state = ctx.client.state().await;
    let initial = state.counter;

    ctx.client.add_to_counter(1).await.await_confirmation().await;
    ctx.client.add_to_counter(1).await.await_confirmation().await;
    let state = ctx.client.state().await;
    assert_eq!(state.counter, initial + 2);

    ctx.client.add_to_counter(10).await.await_confirmation().await;
    let state = ctx.client.state().await;
    assert_eq!(state.counter, initial + 12);

    let ctx = TestContext::new().await;
    ctx.client.add_to_counter(u32::MAX - 100).await.await_confirmation().await;
    let state = ctx.client.state().await;
    assert_eq!(state.counter, u32::MAX - 100);
}

#[tokio::test]
#[serial]
async fn test_message() {
    let ctx = TestContext::new().await;
    let state = ctx.client.state().await;
    assert_eq!(state.message, "init");

    ctx.client.set_message("hello").await.await_confirmation().await;
    let state = ctx.client.state().await;
    assert_eq!(state.message, "hello");

    ctx.client.set_message("").await.await_confirmation().await;
    let state = ctx.client.state().await;
    assert_eq!(state.message, "");

    let large_msg = "x".repeat(10000);
    ctx.client.set_message(&large_msg).await.await_confirmation().await;
    let state = ctx.client.state().await;
    assert_eq!(state.message, large_msg);
}

#[tokio::test]
#[serial]
async fn test_nested_struct() {
    let ctx = TestContext::new().await;

    ctx.client.set_user("Alice", 42).await.await_confirmation().await;
    let state = ctx.client.state().await;
    assert_eq!(state.user_data.name, "Alice");
    assert_eq!(state.user_data.visits, 42);

    // Overwrite
    ctx.client.set_user("Bob", 100).await.await_confirmation().await;
    let state = ctx.client.state().await;
    assert_eq!(state.user_data.name, "Bob");
    assert_eq!(state.user_data.visits, 100);

    // Empty name, zero visits
    ctx.client.set_user("", 0).await.await_confirmation().await;
    let state = ctx.client.state().await;
    assert_eq!(state.user_data.name, "");
    assert_eq!(state.user_data.visits, 0);
}

#[tokio::test]
#[serial]
async fn test_enum_works() {
    let ctx = TestContext::new().await;

    ctx.client.set_enum(TestEnum::AVariation).await.await_confirmation().await;
    let state = ctx.client.state().await;
    assert_eq!(state.enum_test, TestEnum::AVariation);

    ctx.client.set_enum(TestEnum::BVariation).await.await_confirmation().await;
    let state = ctx.client.state().await;
    assert_eq!(state.enum_test, TestEnum::BVariation);
}

#[tokio::test]
#[serial]
async fn test_timestamps_sensible() {
    let ctx = TestContext::new().await;

    ctx.client.set_timestamp().await.await_confirmation().await;

    let state = ctx.client.state().await;
    let current_timestamp =
        SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();

    let timestamp_plus_10_sec = current_timestamp + 10;
    let timestamp_minus_10_sec = current_timestamp - 10;

    assert!(state.current_time < timestamp_plus_10_sec);
    assert!(state.current_time > timestamp_minus_10_sec);
}

#[tokio::test]
#[serial]
async fn test_runtime_functions() {
    let ctx = TestContext::new().await;

    ctx.client.get_sender().await.await_confirmation().await;
    ctx.client.get_block_time().await.await_confirmation().await;
}

#[tokio::test]
#[serial]
async fn test_constructor() {
    ensure_network();
    let client = ContractAbiClient::deploy(
        "../contract/out/contract.wasm",
        "hello from constructor".to_string(),
    )
    .await;

    let state = client.state().await;
    assert_eq!(state.message, "hello from constructor");

    let client = ContractAbiClient::deploy("../contract/out/contract.wasm", "".to_string()).await;
    let state = client.state().await;
    assert_eq!(state.message, "");
}
