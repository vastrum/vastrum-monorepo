use super::*;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::test]
#[serial]
async fn test_nested_kvmap_basic() {
    let ctx = TestContext::new().await;

    let state = ctx.client.state().await;
    assert!(state.nested_kvmap.get(&"k1".to_string()).await.is_none());

    ctx.client.nested_kvmap_create("k1", "n1").await.await_confirmation().await;
    let state = ctx.client.state().await;
    let s = state.nested_kvmap.get(&"k1".to_string()).await.unwrap();
    assert_eq!(s.name, "n1");
    assert_eq!(s.count, 0);
    assert!(s.inner.get(&"x".to_string()).await.is_none());
}

#[tokio::test]
#[serial]
async fn test_nested_kvmap_set_and_get() {
    let ctx = TestContext::new().await;

    ctx.client.nested_kvmap_create("k1", "n1").await.await_confirmation().await;
    ctx.client.nested_kvmap_set("k1", "alice", 100).await.await_confirmation().await;
    ctx.client.nested_kvmap_set("k1", "bob", 50).await.await_confirmation().await;

    let state = ctx.client.state().await;
    let s = state.nested_kvmap.get(&"k1".to_string()).await.unwrap();
    assert_eq!(s.count, 2);
    assert_eq!(s.inner.get(&"alice".to_string()).await, Some(100));
    assert_eq!(s.inner.get(&"bob".to_string()).await, Some(50));
    assert_eq!(s.inner.get(&"nonexistent".to_string()).await, None);
}

#[tokio::test]
#[serial]
async fn test_nested_kvmap_overwrite() {
    let ctx = TestContext::new().await;

    ctx.client.nested_kvmap_create("k1", "n1").await.await_confirmation().await;
    ctx.client.nested_kvmap_set("k1", "alice", 100).await.await_confirmation().await;

    ctx.client.nested_kvmap_set("k1", "alice", 999).await.await_confirmation().await;

    let state = ctx.client.state().await;
    let s = state.nested_kvmap.get(&"k1".to_string()).await.unwrap();
    assert_eq!(s.inner.get(&"alice".to_string()).await, Some(999));
}

#[tokio::test]
#[serial]
async fn test_nested_kvmap_remove() {
    let ctx = TestContext::new().await;

    ctx.client.nested_kvmap_create("k1", "n1").await.await_confirmation().await;
    ctx.client.nested_kvmap_set("k1", "alice", 100).await.await_confirmation().await;
    ctx.client.nested_kvmap_set("k1", "bob", 50).await.await_confirmation().await;

    ctx.client.nested_kvmap_remove("k1", "bob").await.await_confirmation().await;

    let state = ctx.client.state().await;
    let s = state.nested_kvmap.get(&"k1".to_string()).await.unwrap();
    assert_eq!(s.count, 1);
    assert_eq!(s.inner.get(&"alice".to_string()).await, Some(100));
    assert_eq!(s.inner.get(&"bob".to_string()).await, None);
}

#[tokio::test]
#[serial]
async fn test_nested_kvmap_multiple_outers() {
    let ctx = TestContext::new().await;

    ctx.client.nested_kvmap_create("k1", "n1").await.await_confirmation().await;
    ctx.client.nested_kvmap_create("k2", "n2").await.await_confirmation().await;

    ctx.client.nested_kvmap_set("k1", "alice", 100).await.await_confirmation().await;
    ctx.client.nested_kvmap_set("k2", "alice", 200).await.await_confirmation().await;
    ctx.client.nested_kvmap_set("k2", "bob", 300).await.await_confirmation().await;

    let state = ctx.client.state().await;

    let s1 = state.nested_kvmap.get(&"k1".to_string()).await.unwrap();
    assert_eq!(s1.name, "n1");
    assert_eq!(s1.count, 1);
    assert_eq!(s1.inner.get(&"alice".to_string()).await, Some(100));
    assert_eq!(s1.inner.get(&"bob".to_string()).await, None);

    // k2 - alice has a different value, bob is only here
    let s2 = state.nested_kvmap.get(&"k2".to_string()).await.unwrap();
    assert_eq!(s2.name, "n2");
    assert_eq!(s2.count, 2);
    assert_eq!(s2.inner.get(&"alice".to_string()).await, Some(200));
    assert_eq!(s2.inner.get(&"bob".to_string()).await, Some(300));
}

#[tokio::test]
#[serial]
async fn test_nested_kvvec_basic() {
    let ctx = TestContext::new().await;

    ctx.client.nested_kvvec_create("k1", "n1").await.await_confirmation().await;
    let state = ctx.client.state().await;
    let s = state.nested_kvvec.get(&"k1".to_string()).await.unwrap();
    assert_eq!(s.name, "n1");
    assert_eq!(s.inner.length().await, 0);
    assert!(s.inner.is_empty().await);
    assert!(s.inner.get(0).await.is_none());
}

#[tokio::test]
#[serial]
async fn test_nested_kvvec_push_and_get() {
    let ctx = TestContext::new().await;

    ctx.client.nested_kvvec_create("k1", "n1").await.await_confirmation().await;
    ctx.client.nested_kvvec_push("k1", "v1").await.await_confirmation().await;
    ctx.client.nested_kvvec_push("k1", "v2").await.await_confirmation().await;
    ctx.client.nested_kvvec_push("k1", "v3").await.await_confirmation().await;

    let state = ctx.client.state().await;
    let s = state.nested_kvvec.get(&"k1".to_string()).await.unwrap();
    assert_eq!(s.inner.length().await, 3);
    assert_eq!(s.inner.get(0).await, Some("v1".to_string()));
    assert_eq!(s.inner.get(1).await, Some("v2".to_string()));
    assert_eq!(s.inner.get(2).await, Some("v3".to_string()));
    assert_eq!(s.inner.get(3).await, None);
}

#[tokio::test]
#[serial]
async fn test_nested_kvvec_multiple_outers() {
    let ctx = TestContext::new().await;

    ctx.client.nested_kvvec_create("k1", "n1").await.await_confirmation().await;
    ctx.client.nested_kvvec_create("k2", "n2").await.await_confirmation().await;

    ctx.client.nested_kvvec_push("k1", "v1").await.await_confirmation().await;
    ctx.client.nested_kvvec_push("k2", "v2").await.await_confirmation().await;
    ctx.client.nested_kvvec_push("k2", "v3").await.await_confirmation().await;

    let state = ctx.client.state().await;

    let s1 = state.nested_kvvec.get(&"k1".to_string()).await.unwrap();
    assert_eq!(s1.inner.length().await, 1);
    assert_eq!(s1.inner.get(0).await, Some("v1".to_string()));

    let s2 = state.nested_kvvec.get(&"k2".to_string()).await.unwrap();
    assert_eq!(s2.inner.length().await, 2);
    assert_eq!(s2.inner.get(0).await, Some("v2".to_string()));
    assert_eq!(s2.inner.get(1).await, Some("v3".to_string()));
}

#[tokio::test]
#[serial]
async fn test_nested_kvbtree_basic() {
    let ctx = TestContext::new().await;

    assert!(ctx.client.state().await.nested_kvbtree.get(&"k1".to_string()).await.is_none());

    ctx.client.nested_kvbtree_create("k1", "n1").await.await_confirmation().await;
    let state = ctx.client.state().await;
    let s = state.nested_kvbtree.get(&"k1".to_string()).await.unwrap();
    assert_eq!(s.name, "n1");
    assert!(s.inner.is_empty().await);
    assert_eq!(s.inner.length().await, 0);
    assert!(s.inner.get(&100).await.is_none());
    assert!(s.inner.first().await.is_none());
    assert!(s.inner.last().await.is_none());
}

#[tokio::test]
#[serial]
async fn test_nested_kvbtree_insert_and_get() {
    let ctx = TestContext::new().await;

    ctx.client.nested_kvbtree_create("k1", "n1").await.await_confirmation().await;
    ctx.client.nested_kvbtree_insert("k1", 1000, "a").await.await_confirmation().await;
    ctx.client.nested_kvbtree_insert("k1", 500, "b").await.await_confirmation().await;
    ctx.client.nested_kvbtree_insert("k1", 1500, "c").await.await_confirmation().await;

    let state = ctx.client.state().await;
    let s = state.nested_kvbtree.get(&"k1".to_string()).await.unwrap();
    assert_eq!(s.inner.length().await, 3);
    assert_eq!(s.inner.get(&1000).await, Some("a".to_string()));
    assert_eq!(s.inner.get(&500).await, Some("b".to_string()));
    assert_eq!(s.inner.get(&1500).await, Some("c".to_string()));
    assert_eq!(s.inner.get(&9999).await, None);

    assert_eq!(s.inner.first().await.unwrap(), (500, "b".to_string()));
    assert_eq!(s.inner.last().await.unwrap(), (1500, "c".to_string()));
}

#[tokio::test]
#[serial]
async fn test_nested_kvbtree_range_and_last_n() {
    let ctx = TestContext::new().await;

    ctx.client.nested_kvbtree_create("k1", "n1").await.await_confirmation().await;
    for (key, val) in [(100, "v1"), (200, "v2"), (300, "v3"), (400, "v4"), (500, "v5")] {
        ctx.client.nested_kvbtree_insert("k1", key, val).await.await_confirmation().await;
    }

    let state = ctx.client.state().await;
    let s = state.nested_kvbtree.get(&"k1".to_string()).await.unwrap();

    // Range [200, 400)
    let range = s.inner.range(&200, &400).await;
    assert_eq!(range.len(), 2);
    assert_eq!(range[0], (200, "v2".to_string()));
    assert_eq!(range[1], (300, "v3".to_string()));

    let top3 = s.inner.get_descending_entries(3, 0).await;
    assert_eq!(top3.len(), 3);
    assert_eq!(top3[0], (500, "v5".to_string()));
    assert_eq!(top3[1], (400, "v4".to_string()));
    assert_eq!(top3[2], (300, "v3".to_string()));

    let page2 = s.inner.get_descending_entries(2, 2).await;
    assert_eq!(page2.len(), 2);
    assert_eq!(page2[0], (300, "v3".to_string()));
    assert_eq!(page2[1], (200, "v2".to_string()));
}

#[tokio::test]
#[serial]
async fn test_nested_kvbtree_remove() {
    let ctx = TestContext::new().await;

    ctx.client.nested_kvbtree_create("k1", "n1").await.await_confirmation().await;
    ctx.client.nested_kvbtree_insert("k1", 100, "a").await.await_confirmation().await;
    ctx.client.nested_kvbtree_insert("k1", 200, "b").await.await_confirmation().await;

    ctx.client.nested_kvbtree_remove("k1", 100).await.await_confirmation().await;

    let state = ctx.client.state().await;
    let s = state.nested_kvbtree.get(&"k1".to_string()).await.unwrap();
    assert_eq!(s.inner.length().await, 1);
    assert_eq!(s.inner.get(&100).await, None);
    assert_eq!(s.inner.get(&200).await, Some("b".to_string()));
}

#[tokio::test]
#[serial]
async fn test_nested_kvbtree_multiple_outers() {
    let ctx = TestContext::new().await;

    ctx.client.nested_kvbtree_create("k1", "n1").await.await_confirmation().await;
    ctx.client.nested_kvbtree_create("k2", "n2").await.await_confirmation().await;

    ctx.client.nested_kvbtree_insert("k1", 1000, "a").await.await_confirmation().await;
    ctx.client.nested_kvbtree_insert("k2", 500, "a").await.await_confirmation().await;
    ctx.client.nested_kvbtree_insert("k2", 800, "b").await.await_confirmation().await;

    let state = ctx.client.state().await;

    let s1 = state.nested_kvbtree.get(&"k1".to_string()).await.unwrap();
    assert_eq!(s1.inner.length().await, 1);
    assert_eq!(s1.inner.get(&1000).await, Some("a".to_string()));

    let s2 = state.nested_kvbtree.get(&"k2".to_string()).await.unwrap();
    assert_eq!(s2.inner.length().await, 2);
    assert_eq!(s2.inner.get(&500).await, Some("a".to_string()));
    assert_eq!(s2.inner.get(&800).await, Some("b".to_string()));
}

#[tokio::test]
#[serial]
#[ignore]
async fn test_nested_kvbtree_large_offset_pagination() {
    let ctx = TestContext::new().await;

    ctx.client.nested_kvbtree_create("big", "big").await.await_confirmation().await;

    let mut txs = Vec::new();
    for i in 1..=100 {
        txs.push(ctx.client.nested_kvbtree_insert("big", i, format!("v{}", i)).await);
    }
    for tx in txs {
        tx.await_confirmation().await;
    }

    let state = ctx.client.state().await;
    let s = state.nested_kvbtree.get(&"big".to_string()).await.unwrap();
    assert_eq!(s.inner.length().await, 100);

    // Skip top 40, take next 10 → keys 60..=51 descending
    let page = s.inner.get_descending_entries(10, 40).await;
    assert_eq!(page.len(), 10);
    assert_eq!(page[0].0, 60);
    assert_eq!(page[9].0, 51);

    // Skip top 90, take next 10 → keys 10..=1 descending
    let page = s.inner.get_descending_entries(10, 90).await;
    assert_eq!(page.len(), 10);
    assert_eq!(page[0].0, 10);
    assert_eq!(page[9].0, 1);

    // Partial page at end: only 5 entries left
    let page = s.inner.get_descending_entries(10, 95).await;
    assert_eq!(page.len(), 5);
    assert_eq!(page[0].0, 5);
    assert_eq!(page[4].0, 1);

    let page = s.inner.get_descending_entries(10, 100).await;
    assert!(page.is_empty());

    let page = s.inner.get_descending_entries(10, 150).await;
    assert!(page.is_empty());
}

#[tokio::test]
#[serial]
#[ignore]
async fn test_nested_kvbtree_counts_after_remove() {
    let ctx = TestContext::new().await;

    ctx.client.nested_kvbtree_create("rm", "rm").await.await_confirmation().await;

    let mut txs = Vec::new();
    for i in 1..=50 {
        txs.push(ctx.client.nested_kvbtree_insert("rm", i, format!("v{}", i)).await);
    }
    for tx in txs {
        tx.await_confirmation().await;
    }

    // Remove keys 21..=30
    let mut txs = Vec::new();
    for i in 21..=30 {
        txs.push(ctx.client.nested_kvbtree_remove("rm", i).await);
    }
    for tx in txs {
        tx.await_confirmation().await;
    }

    let state = ctx.client.state().await;
    let s = state.nested_kvbtree.get(&"rm".to_string()).await.unwrap();
    assert_eq!(s.inner.length().await, 40);

    // Top 10: keys 50..=41
    let page = s.inner.get_descending_entries(10, 0).await;
    assert_eq!(page.len(), 10);
    assert_eq!(page[0].0, 50);
    assert_eq!(page[9].0, 41);

    // Next 10: keys 40..=31
    let page = s.inner.get_descending_entries(10, 10).await;
    assert_eq!(page.len(), 10);
    assert_eq!(page[0].0, 40);
    assert_eq!(page[9].0, 31);

    // Next 10: keys 20..=11 (gap at 21-30 is gone)
    let page = s.inner.get_descending_entries(10, 20).await;
    assert_eq!(page.len(), 10);
    assert_eq!(page[0].0, 20);
    assert_eq!(page[9].0, 11);

    // Last 10: keys 10..=1
    let page = s.inner.get_descending_entries(10, 30).await;
    assert_eq!(page.len(), 10);
    assert_eq!(page[0].0, 10);
    assert_eq!(page[9].0, 1);

    let page = s.inner.get_descending_entries(10, 40).await;
    assert!(page.is_empty());
}

#[tokio::test]
#[serial]
#[ignore]
async fn test_nested_kvbtree_ascending_entries() {
    let ctx = TestContext::new().await;

    ctx.client.nested_kvbtree_create("asc", "asc").await.await_confirmation().await;

    let mut txs = Vec::new();
    for i in 1..=100 {
        txs.push(ctx.client.nested_kvbtree_insert("asc", i, format!("v{}", i)).await);
    }
    for tx in txs {
        tx.await_confirmation().await;
    }

    let state = ctx.client.state().await;
    let s = state.nested_kvbtree.get(&"asc".to_string()).await.unwrap();

    // First 10 ascending: keys 1..=10
    let page = s.inner.get_ascending_entries(10, 0).await;
    assert_eq!(page.len(), 10);
    assert_eq!(page[0].0, 1);
    assert_eq!(page[9].0, 10);

    // With offset 40: keys 41..=50
    let page = s.inner.get_ascending_entries(10, 40).await;
    assert_eq!(page.len(), 10);
    assert_eq!(page[0].0, 41);
    assert_eq!(page[9].0, 50);

    // With offset 90: keys 91..=100
    let page = s.inner.get_ascending_entries(10, 90).await;
    assert_eq!(page.len(), 10);
    assert_eq!(page[0].0, 91);
    assert_eq!(page[9].0, 100);

    // Partial page at end
    let page = s.inner.get_ascending_entries(10, 95).await;
    assert_eq!(page.len(), 5);
    assert_eq!(page[0].0, 96);
    assert_eq!(page[4].0, 100);

    let page = s.inner.get_ascending_entries(10, 100).await;
    assert!(page.is_empty());

    let page = s.inner.get_ascending_entries(0, 0).await;
    assert!(page.is_empty());
}

#[tokio::test]
#[serial]
#[ignore]
async fn test_nested_kvbtree_ascending_after_remove() {
    let ctx = TestContext::new().await;

    ctx.client.nested_kvbtree_create("asc_rm", "asc_rm").await.await_confirmation().await;

    let mut txs = Vec::new();
    for i in 1..=50 {
        txs.push(ctx.client.nested_kvbtree_insert("asc_rm", i, format!("v{}", i)).await);
    }
    for tx in txs {
        tx.await_confirmation().await;
    }

    // Remove keys 11..=20
    let mut txs = Vec::new();
    for i in 11..=20 {
        txs.push(ctx.client.nested_kvbtree_remove("asc_rm", i).await);
    }
    for tx in txs {
        tx.await_confirmation().await;
    }

    let state = ctx.client.state().await;
    let s = state.nested_kvbtree.get(&"asc_rm".to_string()).await.unwrap();
    assert_eq!(s.inner.length().await, 40);

    // First 10 ascending: keys 1..=10
    let page = s.inner.get_ascending_entries(10, 0).await;
    assert_eq!(page.len(), 10);
    assert_eq!(page[0].0, 1);
    assert_eq!(page[9].0, 10);

    // Next 10: keys 21..=30 (gap at 11-20 is gone)
    let page = s.inner.get_ascending_entries(10, 10).await;
    assert_eq!(page.len(), 10);
    assert_eq!(page[0].0, 21);
    assert_eq!(page[9].0, 30);

    // Next 10: keys 31..=40
    let page = s.inner.get_ascending_entries(10, 20).await;
    assert_eq!(page.len(), 10);
    assert_eq!(page[0].0, 31);
    assert_eq!(page[9].0, 40);

    // Last 10: keys 41..=50
    let page = s.inner.get_ascending_entries(10, 30).await;
    assert_eq!(page.len(), 10);
    assert_eq!(page[0].0, 41);
    assert_eq!(page[9].0, 50);

    let page = s.inner.get_ascending_entries(10, 40).await;
    assert!(page.is_empty());
}

#[tokio::test]
#[serial]
async fn test_nested_kvvecbtree_basic() {
    let ctx = TestContext::new().await;

    assert!(ctx.client.state().await.nested_kvvecbtree.get(&"k1".to_string()).await.is_none());

    ctx.client.nested_kvvecbtree_create("k1", "n1").await.await_confirmation().await;
    let state = ctx.client.state().await;
    let s = state.nested_kvvecbtree.get(&"k1".to_string()).await.unwrap();
    assert_eq!(s.name, "n1");
    assert!(s.inner.is_empty().await);
    assert_eq!(s.inner.length().await, 0);
    assert!(s.inner.get(0).await.is_none());
}

#[tokio::test]
#[serial]
#[ignore]
async fn test_nested_kvvecbtree_push_and_get() {
    let ctx = TestContext::new().await;

    ctx.client.nested_kvvecbtree_create("k1", "n1").await.await_confirmation().await;
    ctx.client
        .nested_kvvecbtree_push("k1", "First title", "First content")
        .await
        .await_confirmation()
        .await;
    sleep(Duration::from_millis(1500)).await;
    ctx.client
        .nested_kvvecbtree_push("k1", "Second title", "Second content")
        .await
        .await_confirmation()
        .await;

    let state = ctx.client.state().await;
    let s = state.nested_kvvecbtree.get(&"k1".to_string()).await.unwrap();
    assert_eq!(s.inner.length().await, 2);

    let item0 = s.inner.get(0).await.unwrap();
    assert_eq!(item0.title, "First title");
    assert_eq!(item0.content, "First content");

    let item1 = s.inner.get(1).await.unwrap();
    assert_eq!(item1.title, "Second title");
    assert_eq!(item1.content, "Second content");

    assert!(s.inner.get(2).await.is_none());
}

#[tokio::test]
#[serial]
#[ignore]
async fn test_nested_kvvecbtree_last_n() {
    let ctx = TestContext::new().await;

    ctx.client.nested_kvvecbtree_create("k1", "n1").await.await_confirmation().await;
    for (title, content) in
        [("Post A", "Content A"), ("Post B", "Content B"), ("Post C", "Content C")]
    {
        ctx.client.nested_kvvecbtree_push("k1", title, content).await.await_confirmation().await;
        sleep(Duration::from_millis(1500)).await;
    }

    let state = ctx.client.state().await;
    let s = state.nested_kvvecbtree.get(&"k1".to_string()).await.unwrap();

    let recent = s.inner.get_descending_entries(2, 0).await;
    assert_eq!(recent.len(), 2);
    assert_eq!(recent[0].content, "Content C");
    assert_eq!(recent[1].content, "Content B");

    let older = s.inner.get_descending_entries(2, 1).await;
    assert_eq!(older.len(), 2);
    assert_eq!(older[0].content, "Content B");
    assert_eq!(older[1].content, "Content A");
}

#[tokio::test]
#[serial]
#[ignore]
async fn test_nested_kvvecbtree_delete() {
    let ctx = TestContext::new().await;

    ctx.client.nested_kvvecbtree_create("k1", "n1").await.await_confirmation().await;
    ctx.client
        .nested_kvvecbtree_push("k1", "keep", "kept content")
        .await
        .await_confirmation()
        .await;
    sleep(Duration::from_millis(1500)).await;
    ctx.client
        .nested_kvvecbtree_push("k1", "delete", "deleted content")
        .await
        .await_confirmation()
        .await;

    ctx.client.nested_kvvecbtree_remove("k1", 1).await.await_confirmation().await;

    let state = ctx.client.state().await;
    let s = state.nested_kvvecbtree.get(&"k1".to_string()).await.unwrap();
    assert_eq!(s.inner.length().await, 1);

    assert!(s.inner.get(1).await.is_none());

    let all = s.inner.get_descending_entries(10, 0).await;
    assert_eq!(all.len(), 1);
    assert_eq!(all[0].title, "keep");
}

#[tokio::test]
#[serial]
#[ignore]
async fn test_nested_kvvecbtree_multiple_outers() {
    let ctx = TestContext::new().await;

    ctx.client.nested_kvvecbtree_create("k1", "n1").await.await_confirmation().await;
    ctx.client.nested_kvvecbtree_create("k2", "n2").await.await_confirmation().await;

    ctx.client.nested_kvvecbtree_push("k1", "Post A", "Content A").await.await_confirmation().await;
    ctx.client.nested_kvvecbtree_push("k2", "Post B", "Content B").await.await_confirmation().await;
    sleep(Duration::from_millis(1500)).await;
    ctx.client.nested_kvvecbtree_push("k2", "Post C", "Content C").await.await_confirmation().await;

    let state = ctx.client.state().await;

    let s1 = state.nested_kvvecbtree.get(&"k1".to_string()).await.unwrap();
    assert_eq!(s1.inner.length().await, 1);
    assert_eq!(s1.inner.get(0).await.unwrap().title, "Post A");

    let s2 = state.nested_kvvecbtree.get(&"k2".to_string()).await.unwrap();
    assert_eq!(s2.inner.length().await, 2);
    assert_eq!(s2.inner.get(0).await.unwrap().title, "Post B");
    assert_eq!(s2.inner.get(1).await.unwrap().title, "Post C");
}
