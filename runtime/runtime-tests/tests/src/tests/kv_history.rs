use super::*;
use vastrum_native_lib::NativeHttpClient;
use vastrum_shared_types::limits::KV_RETENTION_WINDOW;

async fn get_height() -> u64 {
    NativeHttpClient::new().get_latest_block_height().await.unwrap()
}

async fn get_kv_at_height(site_id: Sha256Digest, key: &str, height: u64) -> Option<Vec<u8>> {
    NativeHttpClient::new().get_key_value_at_height(site_id, format!("n.raw.{key}"), height).await
}

#[tokio::test]
#[serial]
async fn test_kv_history_read_at_height() {
    let ctx = TestContext::new().await;

    ctx.client.kv_insert_raw("hkey", vec![1, 2, 3]).await.await_confirmation().await;
    let height_1 = get_height().await;

    ctx.client.kv_insert_raw("hkey", vec![4, 5, 6]).await.await_confirmation().await;
    let height_2 = get_height().await;

    let val = get_kv_at_height(ctx.site_id, "hkey", height_1).await;
    assert_eq!(val, Some(vec![1, 2, 3]), "at height_1 should be [1,2,3]");

    let val = get_kv_at_height(ctx.site_id, "hkey", height_2).await;
    assert_eq!(val, Some(vec![4, 5, 6]), "at height_2 should be [4,5,6]");

    let current = get_height().await;
    let val = get_kv_at_height(ctx.site_id, "hkey", current).await;
    assert_eq!(val, Some(vec![4, 5, 6]), "at current height should be [4,5,6]");
}

#[tokio::test]
#[serial]
async fn test_kv_history_delete_then_query() {
    let ctx = TestContext::new().await;

    ctx.client.kv_insert_raw("dkey", vec![10]).await.await_confirmation().await;
    let height_1 = get_height().await;

    ctx.client.kv_delete_raw("dkey").await.await_confirmation().await;
    let height_2 = get_height().await;

    let val = get_kv_at_height(ctx.site_id, "dkey", height_1).await;
    assert_eq!(val, Some(vec![10]), "at height_1 should be Some([10])");

    let val = get_kv_at_height(ctx.site_id, "dkey", height_2).await;
    assert_eq!(val, None, "at height_2 should be None after delete");
}

#[tokio::test]
#[serial]
async fn test_kv_history_before_first_write() {
    let ctx = TestContext::new().await;

    let height_before = get_height().await;

    ctx.client.kv_insert_raw("nkey", vec![7]).await.await_confirmation().await;
    let height_after = get_height().await;

    let val = get_kv_at_height(ctx.site_id, "nkey", height_before).await;
    assert_eq!(val, None, "before write should be None");

    let val = get_kv_at_height(ctx.site_id, "nkey", height_after).await;
    assert_eq!(val, Some(vec![7]), "after write should be Some([7])");
}

#[tokio::test]
#[serial]
async fn test_kv_history_insert_delete_reinsert() {
    let ctx = TestContext::new().await;

    ctx.client.kv_insert_raw("rkey", vec![1, 2]).await.await_confirmation().await;
    let height_1 = get_height().await;

    ctx.client.kv_delete_raw("rkey").await.await_confirmation().await;
    let height_2 = get_height().await;

    ctx.client.kv_insert_raw("rkey", vec![3, 4]).await.await_confirmation().await;
    let height_3 = get_height().await;

    let val = get_kv_at_height(ctx.site_id, "rkey", height_1).await;
    assert_eq!(val, Some(vec![1, 2]), "at height_1 should be [1, 2]");

    let val = get_kv_at_height(ctx.site_id, "rkey", height_2).await;
    assert_eq!(val, None, "at height_2 should be None after delete");

    let val = get_kv_at_height(ctx.site_id, "rkey", height_3).await;
    assert_eq!(val, Some(vec![3, 4]), "at height_3 should be [3, 4]");
}

#[tokio::test]
#[serial]
async fn test_kv_history_survives_pruning() {
    let ctx = TestContext::new().await;

    ctx.client.kv_insert_raw("prune_key", vec![1, 2, 3]).await.await_confirmation().await;

    // wait for retention window so the initial undo entry gets pruned
    let wait_ms = (KV_RETENTION_WINDOW * 3) as u64;
    tokio::time::sleep(std::time::Duration::from_millis(wait_ms)).await;

    let height_before = get_height().await;

    // overwrite triggers pruning of old entries
    ctx.client.kv_insert_raw("prune_key", vec![4, 5, 6]).await.await_confirmation().await;

    let val = get_kv_at_height(ctx.site_id, "prune_key", height_before).await;
    assert_eq!(val, Some(vec![1, 2, 3]), "value before overwrite should survive pruning");
}

#[tokio::test]
#[serial]
async fn test_kv_history_multiple_overwrites() {
    let ctx = TestContext::new().await;

    ctx.client.kv_insert_raw("mkey", vec![1]).await.await_confirmation().await;
    let h1 = get_height().await;

    ctx.client.kv_insert_raw("mkey", vec![2]).await.await_confirmation().await;
    let h2 = get_height().await;

    ctx.client.kv_insert_raw("mkey", vec![3]).await.await_confirmation().await;
    let h3 = get_height().await;

    let val = get_kv_at_height(ctx.site_id, "mkey", h1).await;
    assert_eq!(val, Some(vec![1]), "at h1 should be [1]");

    let val = get_kv_at_height(ctx.site_id, "mkey", h2).await;
    assert_eq!(val, Some(vec![2]), "at h2 should be [2]");

    let val = get_kv_at_height(ctx.site_id, "mkey", h3).await;
    assert_eq!(val, Some(vec![3]), "at h3 should be [3]");
}

#[tokio::test]
#[serial]
async fn test_kv_history_delete_survives_pruning() {
    let ctx = TestContext::new().await;

    ctx.client.kv_insert_raw("dsp_key", vec![10, 20]).await.await_confirmation().await;

    // wait for retention window so the initial undo entry gets pruned
    let wait_ms = (KV_RETENTION_WINDOW * 3) as u64;
    tokio::time::sleep(std::time::Duration::from_millis(wait_ms)).await;

    let height_before_delete = get_height().await;
    ctx.client.kv_delete_raw("dsp_key").await.await_confirmation().await;

    let val = get_kv_at_height(ctx.site_id, "dsp_key", height_before_delete).await;
    assert_eq!(val, Some(vec![10, 20]), "before delete should be [10,20]");

    let current = get_height().await;
    let val = get_kv_at_height(ctx.site_id, "dsp_key", current).await;
    assert_eq!(val, None, "after delete should be None");
}

#[tokio::test]
#[serial]
async fn test_kv_history_unmodified_key_fallback() {
    let ctx = TestContext::new().await;

    ctx.client.kv_insert_raw("umf_key", vec![42]).await.await_confirmation().await;

    // wait for retention window so the undo entry gets pruned
    let wait_ms = (KV_RETENTION_WINDOW * 3) as u64;
    tokio::time::sleep(std::time::Duration::from_millis(wait_ms)).await;

    let current = get_height().await;
    let val = get_kv_at_height(ctx.site_id, "umf_key", current).await;
    assert_eq!(val, Some(vec![42]), "unmodified key should fallback to current value");
}

#[tokio::test]
#[serial]
async fn test_kv_history_independent_keys() {
    let ctx = TestContext::new().await;

    ctx.client.kv_insert_raw("ik_a", vec![1]).await.await_confirmation().await;
    let h_a = get_height().await;

    ctx.client.kv_insert_raw("ik_b", vec![2]).await.await_confirmation().await;
    let h_b = get_height().await;

    ctx.client.kv_insert_raw("ik_a", vec![3]).await.await_confirmation().await;

    let val = get_kv_at_height(ctx.site_id, "ik_a", h_a).await;
    assert_eq!(val, Some(vec![1]), "key_a at h_a should be [1]");

    let val = get_kv_at_height(ctx.site_id, "ik_b", h_b).await;
    assert_eq!(val, Some(vec![2]), "key_b at h_b should be [2]");

    let current = get_height().await;
    let val = get_kv_at_height(ctx.site_id, "ik_a", current).await;
    assert_eq!(val, Some(vec![3]), "key_a at current should be [3]");
}

#[tokio::test]
#[serial]
async fn test_kv_history_delete_reinsert_after_pruning() {
    let ctx = TestContext::new().await;
    let wait_ms = (KV_RETENTION_WINDOW * 3) as u64;

    ctx.client.kv_insert_raw("drap_key", vec![1, 2]).await.await_confirmation().await;

    // wait to prune the initial undo entry
    tokio::time::sleep(std::time::Duration::from_millis(wait_ms)).await;

    ctx.client.kv_delete_raw("drap_key").await.await_confirmation().await;

    // wait to prune the delete's undo entry
    tokio::time::sleep(std::time::Duration::from_millis(wait_ms)).await;

    let h_before_reinsert = get_height().await;
    ctx.client.kv_insert_raw("drap_key", vec![3, 4]).await.await_confirmation().await;

    let val = get_kv_at_height(ctx.site_id, "drap_key", h_before_reinsert).await;
    assert_eq!(val, None, "before reinsert should be None (key was deleted)");

    let current = get_height().await;
    let val = get_kv_at_height(ctx.site_id, "drap_key", current).await;
    assert_eq!(val, Some(vec![3, 4]), "at current should be [3,4]");
}
