use super::*;

#[tokio::test]
#[serial]
async fn test_kvbtree() {
    let ctx = TestContext::new().await;

    // Empty state
    let state = ctx.client.state().await;
    assert!(state.kvbtree.is_empty().await);
    assert_eq!(state.kvbtree.length().await, 0);
    assert!(state.kvbtree.first().await.is_none());
    assert!(state.kvbtree.last().await.is_none());
    assert!(state.kvbtree.range(&0, &100).await.is_empty());
    assert!(state.kvbtree.get_descending_entries(10, 0).await.is_empty());
    assert!(state.kvbtree.get_ascending_entries(10, 0).await.is_empty());

    // Insert and get
    ctx.client.kvbtree_insert(100, "v100").await.await_confirmation().await;
    ctx.client.kvbtree_insert(50, "v50").await.await_confirmation().await;
    ctx.client.kvbtree_insert(150, "v150").await.await_confirmation().await;

    let state = ctx.client.state().await;
    assert_eq!(state.kvbtree.length().await, 3);
    assert_eq!(state.kvbtree.get(&100).await, Some("v100".to_string()));
    assert_eq!(state.kvbtree.get(&50).await, Some("v50".to_string()));
    assert_eq!(state.kvbtree.get(&999).await, None);

    // Ordering
    assert_eq!(state.kvbtree.first().await.unwrap().0, 50);
    assert_eq!(state.kvbtree.last().await.unwrap().0, 150);

    // Overwrite (no duplicate)
    ctx.client.kvbtree_insert(100, "v100_updated").await.await_confirmation().await;
    let state = ctx.client.state().await;
    assert_eq!(state.kvbtree.get(&100).await, Some("v100_updated".to_string()));
    assert_eq!(state.kvbtree.length().await, 3);

    // Remove
    ctx.client.kvbtree_remove(100).await.await_confirmation().await;
    let state = ctx.client.state().await;
    assert_eq!(state.kvbtree.get(&100).await, None);
    assert_eq!(state.kvbtree.length().await, 2);

    // Remove nonexistent (no error, length unchanged)
    ctx.client.kvbtree_remove(999).await.await_confirmation().await;
    assert_eq!(ctx.client.state().await.kvbtree.length().await, 2);

    // Double remove (no error)
    ctx.client.kvbtree_remove(50).await.await_confirmation().await;
    ctx.client.kvbtree_remove(50).await.await_confirmation().await;
    assert_eq!(ctx.client.state().await.kvbtree.length().await, 1);

    // Remove last → empty, then re-insert
    ctx.client.kvbtree_remove(150).await.await_confirmation().await;
    let state = ctx.client.state().await;
    assert!(state.kvbtree.is_empty().await);
    assert!(state.kvbtree.first().await.is_none());

    ctx.client.kvbtree_insert(42, "reinserted").await.await_confirmation().await;
    let state = ctx.client.state().await;
    assert_eq!(state.kvbtree.length().await, 1);
    assert_eq!(state.kvbtree.get(&42).await, Some("reinserted".to_string()));

    // Edge keys: 0 and u64::MAX
    ctx.client.kvbtree_insert(0, "zero").await.await_confirmation().await;
    ctx.client.kvbtree_insert(u64::MAX, "max").await.await_confirmation().await;
    let state = ctx.client.state().await;
    assert_eq!(state.kvbtree.get(&0).await, Some("zero".to_string()));
    assert_eq!(state.kvbtree.get(&u64::MAX).await, Some("max".to_string()));
    assert_eq!(state.kvbtree.first().await.unwrap().0, 0);
    assert_eq!(state.kvbtree.last().await.unwrap().0, u64::MAX);
}

#[tokio::test]
#[serial]
async fn test_kvbtree_range() {
    let ctx = TestContext::new().await;

    // Range on empty tree
    let state = ctx.client.state().await;
    assert!(state.kvbtree.range(&0, &100).await.is_empty());

    for (k, v) in [(10, "v10"), (20, "v20"), (30, "v30"), (40, "v40"), (50, "v50")] {
        ctx.client.kvbtree_insert(k, v).await.await_confirmation().await;
    }

    let state = ctx.client.state().await;

    // [20, 40) → 20, 30
    let range = state.kvbtree.range(&20, &40).await;
    assert_eq!(range.len(), 2);
    assert_eq!(range[0].0, 20);
    assert_eq!(range[1].0, 30);

    // Full range
    assert_eq!(state.kvbtree.range(&0, &u64::MAX).await.len(), 5);

    // Same start/end (empty)
    assert!(state.kvbtree.range(&30, &30).await.is_empty());

    // Inverted range (empty)
    assert!(state.kvbtree.range(&50, &10).await.is_empty());

    // Outside data
    assert!(state.kvbtree.range(&1, &5).await.is_empty());
    assert!(state.kvbtree.range(&100, &200).await.is_empty());

    // Partial overlaps
    let range = state.kvbtree.range(&5, &25).await;
    assert_eq!(range.len(), 2); // 10, 20

    let range = state.kvbtree.range(&35, &100).await;
    assert_eq!(range.len(), 2); // 40, 50
}

#[tokio::test]
#[serial]
async fn test_kvbtree_descending_entries() {
    let ctx = TestContext::new().await;

    // Empty tree
    assert!(ctx.client.state().await.kvbtree.get_descending_entries(10, 0).await.is_empty());

    for (k, v) in [(10, "v10"), (20, "v20"), (30, "v30"), (40, "v40"), (50, "v50")] {
        ctx.client.kvbtree_insert(k, v).await.await_confirmation().await;
    }

    let state = ctx.client.state().await;

    let last_2 = state.kvbtree.get_descending_entries(2, 0).await;
    assert_eq!(last_2.len(), 2);
    assert_eq!(last_2[0].0, 50);
    assert_eq!(last_2[1].0, 40);

    // With offset
    let with_offset = state.kvbtree.get_descending_entries(2, 1).await;
    assert_eq!(with_offset[0].0, 40);

    // Count > available
    assert_eq!(state.kvbtree.get_descending_entries(100, 0).await.len(), 5);

    // Offset >= count (empty)
    assert!(state.kvbtree.get_descending_entries(3, 5).await.is_empty());

    // Count = 0
    assert!(state.kvbtree.get_descending_entries(0, 0).await.is_empty());
}

#[tokio::test]
#[serial]
async fn test_kvbtree_ascending_entries() {
    let ctx = TestContext::new().await;

    // Empty tree
    assert!(ctx.client.state().await.kvbtree.get_ascending_entries(10, 0).await.is_empty());

    for (k, v) in [(10, "v10"), (20, "v20"), (30, "v30"), (40, "v40"), (50, "v50")] {
        ctx.client.kvbtree_insert(k, v).await.await_confirmation().await;
    }

    let state = ctx.client.state().await;

    // First 2
    let first_2 = state.kvbtree.get_ascending_entries(2, 0).await;
    assert_eq!(first_2.len(), 2);
    assert_eq!(first_2[0].0, 10);
    assert_eq!(first_2[1].0, 20);

    // With offset: skip 1 → start at 20
    let with_offset = state.kvbtree.get_ascending_entries(2, 1).await;
    assert_eq!(with_offset.len(), 2);
    assert_eq!(with_offset[0].0, 20);
    assert_eq!(with_offset[1].0, 30);

    // Count > available
    assert_eq!(state.kvbtree.get_ascending_entries(100, 0).await.len(), 5);

    // Offset >= total (empty)
    assert!(state.kvbtree.get_ascending_entries(3, 5).await.is_empty());

    // Count = 0
    assert!(state.kvbtree.get_ascending_entries(0, 0).await.is_empty());
}

#[tokio::test]
#[serial]
async fn test_kvbtree_large_insert_and_ordering() {
    let ctx = TestContext::new().await;

    let n = 50u64;
    for i in 0..n {
        ctx.client.kvbtree_insert(i, &format!("v{i}")).await.await_confirmation().await;
    }

    let state = ctx.client.state().await;

    // Length
    assert_eq!(state.kvbtree.length().await, n);

    // Extremes
    assert_eq!(state.kvbtree.first().await.unwrap().0, 0);
    assert_eq!(state.kvbtree.last().await.unwrap().0, n - 1);

    // Every key retrievable with correct value
    for i in 0..n {
        assert_eq!(
            state.kvbtree.get(&i).await,
            Some(format!("v{i}")),
            "get({i}) should return v{i}"
        );
    }

    // Ascending order
    let asc = state.kvbtree.get_ascending_entries(n as usize, 0).await;
    assert_eq!(asc.len(), n as usize);
    for (idx, (key, val)) in asc.iter().enumerate() {
        assert_eq!(*key, idx as u64);
        assert_eq!(val, &format!("v{idx}"));
    }

    // Descending order
    let desc = state.kvbtree.get_descending_entries(n as usize, 0).await;
    assert_eq!(desc.len(), n as usize);
    for (idx, (key, _)) in desc.iter().enumerate() {
        assert_eq!(*key, n - 1 - idx as u64);
    }
}

#[tokio::test]
#[serial]
async fn test_kvbtree_large_pagination() {
    let ctx = TestContext::new().await;

    let n = 50u64;
    for i in 0..n {
        ctx.client.kvbtree_insert(i, &format!("v{i}")).await.await_confirmation().await;
    }

    let state = ctx.client.state().await;
    let page_size = 10usize;

    // Paginate ascending
    let mut all_asc_keys = Vec::new();
    for page in 0..5usize {
        let entries = state.kvbtree.get_ascending_entries(page_size, page * page_size).await;
        assert_eq!(
            entries.len(),
            page_size,
            "ascending page {page} should have {page_size} entries"
        );
        for (key, _) in &entries {
            all_asc_keys.push(*key);
        }
    }
    // No duplicates, no gaps
    assert_eq!(all_asc_keys.len(), n as usize);
    for (idx, key) in all_asc_keys.iter().enumerate() {
        assert_eq!(*key, idx as u64, "ascending pagination gap/dup at index {idx}");
    }

    // Paginate descending
    let mut all_desc_keys = Vec::new();
    for page in 0..5usize {
        let entries = state.kvbtree.get_descending_entries(page_size, page * page_size).await;
        assert_eq!(
            entries.len(),
            page_size,
            "descending page {page} should have {page_size} entries"
        );
        for (key, _) in &entries {
            all_desc_keys.push(*key);
        }
    }
    assert_eq!(all_desc_keys.len(), n as usize);
    for (idx, key) in all_desc_keys.iter().enumerate() {
        assert_eq!(*key, n - 1 - idx as u64, "descending pagination gap/dup at index {idx}");
    }

    // Range queries spanning node boundaries
    let range = state.kvbtree.range(&10, &40).await;
    assert_eq!(range.len(), 30); // keys 10..40
    for (idx, (key, _)) in range.iter().enumerate() {
        assert_eq!(*key, 10 + idx as u64);
    }

    let range = state.kvbtree.range(&0, &n).await;
    assert_eq!(range.len(), n as usize);
}

#[tokio::test]
#[serial]
async fn test_kvbtree_large_delete_and_rebalance() {
    let ctx = TestContext::new().await;

    let n = 50u64;
    for i in 0..n {
        ctx.client.kvbtree_insert(i, &format!("v{i}")).await.await_confirmation().await;
    }

    // Delete even keys 0,2,4,...,48 (25 deletions: all evens in 0..50)
    // then also delete 1,3,5,7,9 (5 more) = 30 total deletions
    let mut deleted = Vec::new();
    for i in (0..n).filter(|x| x % 2 == 0) {
        ctx.client.kvbtree_remove(i).await.await_confirmation().await;
        deleted.push(i);
    }
    for i in [1u64, 3, 5, 7, 9] {
        ctx.client.kvbtree_remove(i).await.await_confirmation().await;
        deleted.push(i);
    }

    let remaining: Vec<u64> = (0..n).filter(|x| !deleted.contains(x)).collect();
    let state = ctx.client.state().await;

    assert_eq!(state.kvbtree.length().await, remaining.len() as u64);

    // All deleted keys return None
    for &k in &deleted {
        assert_eq!(state.kvbtree.get(&k).await, None, "deleted key {k} should be None");
    }

    // All remaining keys return correct value
    for &k in &remaining {
        assert_eq!(
            state.kvbtree.get(&k).await,
            Some(format!("v{k}")),
            "remaining key {k} should still exist"
        );
    }

    // Ascending ordering is correct on sparse tree
    let asc = state.kvbtree.get_ascending_entries(remaining.len(), 0).await;
    assert_eq!(asc.len(), remaining.len());
    for (idx, (key, _)) in asc.iter().enumerate() {
        assert_eq!(*key, remaining[idx], "ascending order broken at index {idx}");
    }

    // Range query on sparse tree
    let range = state.kvbtree.range(&10, &40).await;
    let expected_range: Vec<u64> =
        remaining.iter().copied().filter(|&k| k >= 10 && k < 40).collect();
    assert_eq!(range.len(), expected_range.len());
    for (idx, (key, _)) in range.iter().enumerate() {
        assert_eq!(*key, expected_range[idx]);
    }

    // Re-insert some deleted keys and verify tree still works
    for &k in deleted.iter().take(10) {
        ctx.client.kvbtree_insert(k, &format!("v{k}_reinserted")).await.await_confirmation().await;
    }

    let state = ctx.client.state().await;
    assert_eq!(state.kvbtree.length().await, (remaining.len() + 10) as u64);

    // Verify re-inserted keys
    for &k in deleted.iter().take(10) {
        assert_eq!(state.kvbtree.get(&k).await, Some(format!("v{k}_reinserted")));
    }

    // Verify full ascending order after re-insertion
    let total = state.kvbtree.length().await as usize;
    let asc = state.kvbtree.get_ascending_entries(total, 0).await;
    assert_eq!(asc.len(), total);
    for window in asc.windows(2) {
        assert!(window[0].0 < window[1].0, "ordering broken: {} >= {}", window[0].0, window[1].0);
    }
}

/*
async fn pipeline_inserts(
    client: &ContractAbiClient,
    entries: impl IntoIterator<Item = (u64, String)>,
) {
    let mut pending = Vec::new();
    for (k, v) in entries {
        pending.push(client.kvbtree_insert(k, &v).await);
    }
    for tx in pending {
        tx.await_confirmation().await;
    }
}

async fn pipeline_removes(client: &ContractAbiClient, keys: impl IntoIterator<Item = u64>) {
    let mut pending = Vec::new();
    for k in keys {
        pending.push(client.kvbtree_remove(k).await);
    }
    for tx in pending {
        tx.await_confirmation().await;
    }
}
#[tokio::test]
#[serial]
#[ignore]
async fn test_kvbtree_stress_1k() {
    let ctx = TestContext::new().await;
    let n = 1_000u64;

    // Pipeline insert 1,000 entries (2-3 level deep tree with MAX_KEYS=31)
    pipeline_inserts(&ctx.client, (0..n).map(|i| (i, format!("v{i}")))).await;

    let state = ctx.client.state().await;

    // Length
    assert_eq!(state.kvbtree.length().await, n);

    // Extremes
    assert_eq!(state.kvbtree.first().await.unwrap().0, 0);
    assert_eq!(state.kvbtree.last().await.unwrap().0, n - 1);

    // Every key retrievable with correct value
    for i in 0..n {
        assert_eq!(
            state.kvbtree.get(&i).await,
            Some(format!("v{i}")),
            "get({i}) should return v{i}"
        );
    }

    // Ascending order - strict ordering via windows(2)
    let asc = state.kvbtree.get_ascending_entries(n as usize, 0).await;
    assert_eq!(asc.len(), n as usize);
    assert_eq!(asc[0].0, 0);
    assert_eq!(asc[asc.len() - 1].0, n - 1);
    for window in asc.windows(2) {
        assert!(window[0].0 < window[1].0, "ascending order broken: {} >= {}", window[0].0, window[1].0);
    }

    // Descending order
    let desc = state.kvbtree.get_descending_entries(n as usize, 0).await;
    assert_eq!(desc.len(), n as usize);
    for window in desc.windows(2) {
        assert!(window[0].0 > window[1].0, "descending order broken: {} <= {}", window[0].0, window[1].0);
    }

    // Paginate ascending (page_size=100, 10 pages)
    let page_size = 100usize;
    let mut all_keys = Vec::new();
    for page in 0..10usize {
        let entries = state
            .kvbtree
            .get_ascending_entries(page_size, page * page_size)
            .await;
        assert_eq!(entries.len(), page_size, "ascending page {page} should have {page_size} entries");
        for (key, _) in &entries {
            all_keys.push(*key);
        }
    }
    assert_eq!(all_keys.len(), n as usize);
    for window in all_keys.windows(2) {
        assert!(window[0] < window[1], "pagination gap/dup: {} >= {}", window[0], window[1]);
    }

    // Range queries
    let range = state.kvbtree.range(&0, &500).await;
    assert_eq!(range.len(), 500);

    let range = state.kvbtree.range(&300, &700).await;
    assert_eq!(range.len(), 400);

    // Delete 500 entries (all even keys) - cascading merges
    pipeline_removes(&ctx.client, (0..n).filter(|k| k % 2 == 0)).await;

    let state = ctx.client.state().await;
    assert_eq!(state.kvbtree.length().await, 500);

    // Ascending order of remaining 500 odd keys
    let asc = state.kvbtree.get_ascending_entries(500, 0).await;
    assert_eq!(asc.len(), 500);
    assert_eq!(asc[0].0, 1);
    assert_eq!(asc[asc.len() - 1].0, n - 1); // 999
    for window in asc.windows(2) {
        assert!(window[0].0 < window[1].0, "odd-key ordering broken: {} >= {}", window[0].0, window[1].0);
    }

    // Spot-check deleted keys return None
    for i in (0..n).step_by(20) {
        assert_eq!(state.kvbtree.get(&i).await, None, "deleted even key {i} should be None");
    }test_kvbtree_stress_1k

    // Range [100, 500) - odd keys: 101,103,...,499
    let range = state.kvbtree.range(&100, &500).await;
    let expected_count = (100..500u64).filter(|k| k % 2 != 0).count();
    assert_eq!(range.len(), expected_count);

    // Re-insert 200 entries (keys 1_000..1_200) - splits on previously-merged tree
    pipeline_inserts(&ctx.client, (1_000..1_200u64).map(|i| (i, format!("v{i}")))).await;

    let state = ctx.client.state().await;
    assert_eq!(state.kvbtree.length().await, 700);

    // First and last correct after mixed survivors + new inserts
    assert_eq!(state.kvbtree.first().await.unwrap().0, 1);
    assert_eq!(state.kvbtree.last().await.unwrap().0, 1_199);

    // Full ascending order via windows(2)
    let asc = state.kvbtree.get_ascending_entries(700, 0).await;
    assert_eq!(asc.len(), 700);
    for window in asc.windows(2) {
        assert!(window[0].0 < window[1].0, "final ordering broken: {} >= {}", window[0].0, window[1].0);
    }
}
*/
