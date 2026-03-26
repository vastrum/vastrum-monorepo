use super::*;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::test]
#[serial]
async fn test_kvvecbtree_create_and_get() {
    let ctx = TestContext::new().await;

    let state = ctx.client.state().await;
    assert!(state.kvvecbtree.is_empty().await);
    assert_eq!(state.kvvecbtree.length().await, 0);
    assert!(state.kvvecbtree.get(0).await.is_none());
    assert!(state.kvvecbtree.get(999).await.is_none());

    ctx.client.kvvecbtree_push("Post A", "Content A", "alice").await.await_confirmation().await;
    ctx.client.kvvecbtree_push("Post B", "Content B", "bob").await.await_confirmation().await;

    let state = ctx.client.state().await;
    assert_eq!(state.kvvecbtree.length().await, 2);

    let item0 = state.kvvecbtree.get(0).await.unwrap();
    assert_eq!(item0.title, "Post A");
    assert_eq!(item0.author, "alice");
    assert_eq!(item0.reply_count, 0);

    let item1 = state.kvvecbtree.get(1).await.unwrap();
    assert_eq!(item1.title, "Post B");

    assert!(state.kvvecbtree.get(999).await.is_none());
}

#[tokio::test]
#[serial]
#[ignore]
async fn test_kvvecbtree_ordering_and_pagination() {
    let ctx = TestContext::new().await;

    ctx.client.kvvecbtree_push("Post A", "Content A", "alice").await.await_confirmation().await;
    sleep(Duration::from_millis(1500)).await;

    ctx.client.kvvecbtree_push("Post B", "Content B", "bob").await.await_confirmation().await;
    sleep(Duration::from_millis(1500)).await;

    ctx.client.kvvecbtree_push("Post C", "Content C", "charlie").await.await_confirmation().await;

    let state = ctx.client.state().await;
    let item0 = state.kvvecbtree.get(0).await.unwrap();
    let item2 = state.kvvecbtree.get(2).await.unwrap();

    let range = state.kvvecbtree.range(&item0.last_bump_time, &item2.last_bump_time).await;
    assert_eq!(range.len(), 2); // exclusive end

    let recent = state.kvvecbtree.get_descending_entries(3, 0).await;
    assert_eq!(recent.len(), 3);
    assert_eq!(recent[0].title, "Post C");
    assert_eq!(recent[1].title, "Post B");
    assert_eq!(recent[2].title, "Post A");

    let page2 = state.kvvecbtree.get_descending_entries(2, 1).await;
    assert_eq!(page2.len(), 2);
    assert_eq!(page2[0].title, "Post B");

    let asc = state.kvvecbtree.get_ascending_entries(3, 0).await;
    assert_eq!(asc.len(), 3);
    assert_eq!(asc[0].title, "Post A");
    assert_eq!(asc[1].title, "Post B");
    assert_eq!(asc[2].title, "Post C");

    let asc_page = state.kvvecbtree.get_ascending_entries(2, 1).await;
    assert_eq!(asc_page.len(), 2);
    assert_eq!(asc_page[0].title, "Post B");
    assert_eq!(asc_page[1].title, "Post C");

    // Pagination overflow: count larger than available
    let big_count = state.kvvecbtree.get_descending_entries(100, 0).await;
    assert_eq!(big_count.len(), 3);

    let past_end = state.kvvecbtree.get_descending_entries(2, 10).await;
    assert!(past_end.is_empty());

    let zero = state.kvvecbtree.get_descending_entries(0, 0).await;
    assert!(zero.is_empty());

    // Empty range (start == end, exclusive end)
    let empty = state.kvvecbtree.range(&item0.last_bump_time, &item0.last_bump_time).await;
    assert!(empty.is_empty());

    let all = state.kvvecbtree.range(&0, &u64::MAX).await;
    assert_eq!(all.len(), 3);

    let future = state.kvvecbtree.range(&(item2.last_bump_time + 1000), &u64::MAX).await;
    assert!(future.is_empty());
}

#[tokio::test]
#[serial]
#[ignore]
async fn test_kvvecbtree_update_bumps_to_top() {
    let ctx = TestContext::new().await;

    ctx.client.kvvecbtree_push("Old post", "Content A", "alice").await.await_confirmation().await;
    sleep(Duration::from_millis(1500)).await;

    ctx.client.kvvecbtree_push("New post", "Content B", "bob").await.await_confirmation().await;

    let state = ctx.client.state().await;
    let recent = state.kvvecbtree.get_descending_entries(2, 0).await;
    assert_eq!(recent[0].title, "New post");

    let old_item = state.kvvecbtree.get(0).await.unwrap();
    let old_bump = old_item.last_bump_time;
    sleep(Duration::from_millis(1500)).await;

    ctx.client.kvvecbtree_update(0).await.await_confirmation().await;

    let state = ctx.client.state().await;
    let item = state.kvvecbtree.get(0).await.unwrap();
    assert_eq!(item.reply_count, 1);
    assert!(item.last_bump_time > old_bump);

    let recent = state.kvvecbtree.get_descending_entries(2, 0).await;
    assert_eq!(recent[0].title, "Old post"); // bumped to top
}

#[tokio::test]
#[serial]
#[ignore]
async fn test_kvvecbtree_multiple_updates() {
    let ctx = TestContext::new().await;

    ctx.client.kvvecbtree_push("My post", "Some content", "alice").await.await_confirmation().await;

    let state = ctx.client.state().await;
    let item = state.kvvecbtree.get(0).await.unwrap();
    let bump0 = item.last_bump_time;
    sleep(Duration::from_millis(1500)).await;

    ctx.client.kvvecbtree_update(0).await.await_confirmation().await;
    let state = ctx.client.state().await;
    let item = state.kvvecbtree.get(0).await.unwrap();
    assert_eq!(item.reply_count, 1);
    assert!(item.last_bump_time > bump0);

    let bump1 = item.last_bump_time;
    sleep(Duration::from_millis(1500)).await;
    ctx.client.kvvecbtree_update(0).await.await_confirmation().await;
    let state = ctx.client.state().await;
    let item = state.kvvecbtree.get(0).await.unwrap();
    assert_eq!(item.reply_count, 2);
    assert!(item.last_bump_time > bump1);
}

#[tokio::test]
#[serial]
async fn test_kvvecbtree_update_nonexistent() {
    let ctx = TestContext::new().await;

    ctx.client.kvvecbtree_push("My post", "Some content", "alice").await.await_confirmation().await;

    ctx.client.kvvecbtree_update(999).await.await_confirmation().await;

    let state = ctx.client.state().await;
    assert_eq!(state.kvvecbtree.length().await, 1);
    let item = state.kvvecbtree.get(0).await.unwrap();
    assert_eq!(item.reply_count, 0);
}

#[tokio::test]
#[serial]
async fn test_kvvecbtree_remove() {
    let ctx = TestContext::new().await;

    ctx.client.kvvecbtree_push("My post", "Some content", "alice").await.await_confirmation().await;

    let state = ctx.client.state().await;
    assert!(state.kvvecbtree.get(0).await.is_some());

    ctx.client.kvvecbtree_remove(0).await.await_confirmation().await;

    let state = ctx.client.state().await;
    assert_eq!(state.kvvecbtree.length().await, 0);
    assert!(state.kvvecbtree.get(0).await.is_none());

    ctx.client.kvvecbtree_remove(0).await.await_confirmation().await;
}

#[tokio::test]
#[serial]
async fn test_kvvecbtree_duplicate_timestamps() {
    let ctx = TestContext::new().await;

    // Two items without sleep  may share timestamp
    ctx.client.kvvecbtree_push("Dup 1", "Content 1", "alice").await.await_confirmation().await;
    ctx.client.kvvecbtree_push("Dup 2", "Content 2", "bob").await.await_confirmation().await;

    let state = ctx.client.state().await;
    let item0 = state.kvvecbtree.get(0).await.unwrap();
    let item1 = state.kvvecbtree.get(1).await.unwrap();
    assert_eq!(item0.title, "Dup 1");
    assert_eq!(item1.title, "Dup 2");
}

#[tokio::test]
#[serial]
async fn test_kvvecbtree_empty_fields() {
    let ctx = TestContext::new().await;

    ctx.client.kvvecbtree_push("", "", "").await.await_confirmation().await;

    let state = ctx.client.state().await;
    let item = state.kvvecbtree.get(0).await.unwrap();
    assert_eq!(item.title, "");
    assert_eq!(item.content, "");
    assert_eq!(item.author, "");
}

#[tokio::test]
#[serial]
async fn test_kvvecbtree_remove_and_reinsert() {
    let ctx = TestContext::new().await;

    ctx.client.kvvecbtree_push("Post A", "Content A", "alice").await.await_confirmation().await;
    ctx.client.kvvecbtree_push("Post B", "Content B", "bob").await.await_confirmation().await;
    ctx.client.kvvecbtree_push("Post C", "Content C", "charlie").await.await_confirmation().await;

    ctx.client.kvvecbtree_remove(0).await.await_confirmation().await;
    ctx.client.kvvecbtree_remove(1).await.await_confirmation().await;
    ctx.client.kvvecbtree_remove(2).await.await_confirmation().await;

    let state = ctx.client.state().await;
    assert!(state.kvvecbtree.is_empty().await);
    assert_eq!(state.kvvecbtree.length().await, 0);
    assert!(state.kvvecbtree.get_descending_entries(10, 0).await.is_empty());
    assert!(state.kvvecbtree.range(&0, &u64::MAX).await.is_empty());

    assert!(state.kvvecbtree.get(0).await.is_none());

    ctx.client.kvvecbtree_push("Post D", "Content D", "dave").await.await_confirmation().await;
    let state = ctx.client.state().await;
    assert_eq!(state.kvvecbtree.length().await, 1);
    let new_item = state.kvvecbtree.get(3).await.unwrap();
    assert_eq!(new_item.title, "Post D");
}

#[tokio::test]
#[serial]
#[ignore]
async fn test_kvvecbtree_remove_middle_ordering() {
    let ctx = TestContext::new().await;

    ctx.client.kvvecbtree_push("Post A", "Content A", "alice").await.await_confirmation().await;
    sleep(Duration::from_millis(1500)).await;
    ctx.client.kvvecbtree_push("Post B", "Content B", "bob").await.await_confirmation().await;
    sleep(Duration::from_millis(1500)).await;
    ctx.client.kvvecbtree_push("Post C", "Content C", "charlie").await.await_confirmation().await;

    // Delete middle item (Post B)
    ctx.client.kvvecbtree_remove(1).await.await_confirmation().await;

    let state = ctx.client.state().await;
    assert_eq!(state.kvvecbtree.length().await, 2);

    // Descending: newest first
    let desc = state.kvvecbtree.get_descending_entries(3, 0).await;
    assert_eq!(desc.len(), 2);
    assert_eq!(desc[0].title, "Post C");
    assert_eq!(desc[1].title, "Post A");

    // Ascending: oldest first
    let asc = state.kvvecbtree.get_ascending_entries(3, 0).await;
    assert_eq!(asc.len(), 2);
    assert_eq!(asc[0].title, "Post A");
    assert_eq!(asc[1].title, "Post C");

    // Slot 1 (Post B) is gone, slots 0 and 2 remain
    assert!(state.kvvecbtree.get(1).await.is_none());
    assert_eq!(state.kvvecbtree.get(0).await.unwrap().title, "Post A");
    assert_eq!(state.kvvecbtree.get(2).await.unwrap().title, "Post C");
}
