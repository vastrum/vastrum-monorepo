use super::*;
use vastrum_native_lib::NativeHttpClient;
use vastrum_shared_types::compression::brotli::brotli_compress_html;

#[tokio::test]
#[serial]
async fn test_add_page_serves_html() {
    let ctx = TestContext::new().await;

    let html = "<html><body>hello from contract</body></html>";
    ctx.client.add_page("", brotli_compress_html(html)).await.await_confirmation().await;

    let http = NativeHttpClient::new();
    let content = http
        .get_page_content(ctx.site_id.to_string(), "".to_string())
        .await
        .expect("get_page failed");

    assert_eq!(content, html);
}

#[tokio::test]
#[serial]
async fn test_add_page_named_route() {
    let ctx = TestContext::new().await;

    let html = "<html><body>about page</body></html>";
    ctx.client.add_page("about", brotli_compress_html(html)).await.await_confirmation().await;

    let http = NativeHttpClient::new();
    let content = http
        .get_page_content(ctx.site_id.to_string(), "about".to_string())
        .await
        .expect("get_page about failed");

    assert_eq!(content, html);
}

#[tokio::test]
#[serial]
async fn test_add_page_1mb_html() {
    let ctx = TestContext::new().await;

    let body = "x".repeat(1_000_000);
    let html = format!("<html><body>{}</body></html>", body);
    ctx.client.add_page("big", brotli_compress_html(&html)).await.await_confirmation().await;

    let http = NativeHttpClient::new();
    let content = http
        .get_page_content(ctx.site_id.to_string(), "big".to_string())
        .await
        .expect("get_page 1MB failed");

    assert_eq!(content, html);
}
/*
#[tokio::test]
#[serial]
async fn test_add_page_10mb_html() {
    let ctx = TestContext::new().await;

    let body = "x".repeat(10_000_000);
    let html = format!("<html><body>{}</body></html>", body);
    ctx.client.add_page("huge", &html).await.await_confirmation().await;

    let http = NativeHttpClient::new();
    let content = http
        .get_page_content(ctx.site_id.to_string(), "huge".to_string())
        .await
        .expect("get_page 10MB failed");

    assert_eq!(content, html);
}
*/
