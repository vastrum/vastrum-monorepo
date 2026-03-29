use super::*;
use vastrum_native_lib::NativeHttpClient;
use vastrum_native_lib::deployers::deploy::register_domain;

#[tokio::test]
#[serial]
async fn test_resolve_domain() {
    let ctx = TestContext::new().await;
    let http = NativeHttpClient::new();

    register_domain(ctx.site_id, "resolvetest").await.await_confirmation().await;

    let resolved = http.resolve_domain("resolvetest".to_string()).await.unwrap();
    assert_eq!(resolved, Some(ctx.site_id));
}

#[tokio::test]
#[serial]
async fn test_resolve_nonexistent_domain() {
    ensure_network();
    let http = NativeHttpClient::new();

    let resolved = http.resolve_domain("this-domain-does-not-exist".to_string()).await.unwrap();
    assert_eq!(resolved, None);
}

#[tokio::test]
#[serial]
async fn test_domain_first_wins() {
    let ctx_a = TestContext::new().await;
    let ctx_b = TestContext::new().await;
    let http = NativeHttpClient::new();

    register_domain(ctx_a.site_id, "firstwins").await.await_confirmation().await;
    register_domain(ctx_b.site_id, "firstwins").await.await_confirmation().await;

    let resolved = http.resolve_domain("firstwins".to_string()).await.unwrap();
    assert_eq!(resolved, Some(ctx_a.site_id));
}

// #[tokio::test]
// #[serial]
//TODO: renable test when this rejection is added
// async fn test_reject_domain_that_looks_like_site_id() {
//     let ctx = TestContext::new().await;
//     let http = NativeHttpClient::new();
//
//     register_domain(ctx.site_id, ctx.site_id.to_string()).await.await_confirmation().await;
//
//     let resolved = http.resolve_domain(ctx.site_id.to_string()).await.unwrap();
//     assert_eq!(resolved, None);
// }
