use super::*;

#[tokio::test]
#[serial]
async fn test_authentication() {
    let ctx = TestContext::new().await;
    let expected_pub_key = ctx.account_key.public_key();

    ctx.client.auth_record_sender().await.await_confirmation().await;
    let state = ctx.client.state().await;
    assert_eq!(state.last_authenticated_sender, expected_pub_key);

    //check that another account leads to different msg_sender
    let other_account = ed25519::PrivateKey::from_seed(222);
    let other_pub_key = other_account.public_key();
    let other_client = ContractAbiClient::new(ctx.site_id).with_account_key(other_account);
    other_client.auth_record_sender().await.await_confirmation().await;

    let state = other_client.state().await;
    assert_eq!(state.last_authenticated_sender, other_pub_key);
}
