const MAX_STALE_RETRIES: usize = 4;

pub struct KvBTree<K, V> {
    nonce: u64,
    client: Arc<RpcClient>,
    _phantom: PhantomData<(K, V)>,
}

impl<K, V> Clone for KvBTree<K, V> {
    fn clone(&self) -> Self {
        return Self { nonce: self.nonce, client: self.client.clone(), _phantom: PhantomData };
    }
}

impl<K, V> fmt::Debug for KvBTree<K, V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        return f.debug_struct("KvBTree").field("nonce", &self.nonce).finish_non_exhaustive();
    }
}

impl<K, V> BorshDeserialize for KvBTree<K, V> {
    fn deserialize_reader<R: io::Read>(reader: &mut R) -> io::Result<Self> {
        //to pass rpc client to child kv data types as they need it to get()
        //very complicated but works
        let nonce = u64::deserialize_reader(reader)?;
        let client = crate::get_deser_client();
        return Ok(Self { nonce, client, _phantom: PhantomData });
    }
}

impl<K, V> KvBTree<K, V>
where
    K: Ord + Clone + BorshDeserialize,
    V: Clone + BorshDeserialize,
{
    pub fn new(nonce: u64, client: Arc<RpcClient>) -> Self {
        return Self { nonce, client, _phantom: PhantomData };
    }

    pub async fn length(&self) -> u64 {
        return self.load_directory().await.len;
    }

    pub async fn is_empty(&self) -> bool {
        return self.load_directory().await.leaves.is_empty();
    }

    pub async fn get(&self, key: &K) -> Option<V> {
        for _ in 0..MAX_STALE_RETRIES {
            let directory = self.load_directory().await;
            let index = locate(&directory, key)?;

            let Some(leaf) = self.load_leaf(directory.leaves[index].leaf_id).await else {
                continue;
            };
            if !leaf_owns(&leaf, key) {
                continue;
            }
            let entry_index = leaf.entries.binary_search_by(|entry| entry.key.cmp(key)).ok()?;
            return Some(leaf.entries[entry_index].value.clone());
        }
        return None;
    }

    pub async fn first(&self) -> Option<(K, V)> {
        return self.get_ascending_entries(1, 0).await.pop();
    }

    pub async fn last(&self) -> Option<(K, V)> {
        return self.get_descending_entries(1, 0).await.pop();
    }

    pub async fn range(&self, start: &K, end: &K) -> Vec<(K, V)> {
        for _ in 0..MAX_STALE_RETRIES {
            let directory = self.load_directory().await;
            let Some(first_index) = locate(&directory, start) else { return Vec::new() };

            let mut selected = Vec::new();
            for index in first_index..directory.leaves.len() {
                let leaf_starts_past_the_range = match &directory.leaves[index].low {
                    Some(low) => low >= end,
                    None => false,
                };
                if leaf_starts_past_the_range {
                    break;
                }
                selected.push(index);
            }

            let Some(leaves) = self.fetch_leaves(&directory, &selected).await else { continue };

            let mut results = Vec::new();
            for leaf in leaves {
                for entry in leaf.entries {
                    if entry.key < *start {
                        continue;
                    }
                    if entry.key >= *end {
                        return results;
                    }
                    results.push((entry.key, entry.value));
                }
            }
            return results;
        }
        return Vec::new();
    }

    pub async fn get_ascending_entries(&self, count: usize, offset: usize) -> Vec<(K, V)> {
        return self.paginate(count, offset, false).await;
    }

    pub async fn get_descending_entries(&self, count: usize, offset: usize) -> Vec<(K, V)> {
        return self.paginate(count, offset, true).await;
    }
}

impl<K, V> KvBTree<K, V>
where
    K: Ord + Clone + BorshDeserialize,
    V: Clone + BorshDeserialize,
{
    fn directory_key(&self) -> String {
        return format!("n.{}.meta", self.nonce);
    }

    fn leaf_key(&self, leaf_id: u64) -> String {
        return format!("n.{}.leaf.{}", self.nonce, leaf_id);
    }

    async fn load_directory(&self) -> Directory<K> {
        let empty = Directory { next_leaf_id: 0, len: 0, leaves: Vec::new() };
        let Some(bytes) = self.client.get_key_value(self.directory_key()).await else {
            return empty;
        };
        if bytes.is_empty() {
            return empty;
        }
        let decoded = crate::with_deser_client(&self.client, || borsh::from_slice(&bytes).ok());
        return decoded.unwrap_or(empty);
    }

    async fn load_leaf(&self, leaf_id: u64) -> Option<LeafNode<K, V>> {
        let bytes = self.client.get_key_value(self.leaf_key(leaf_id)).await?;
        if bytes.is_empty() {
            return None;
        }
        return crate::with_deser_client(&self.client, || borsh::from_slice(&bytes).ok());
    }

    async fn fetch_leaves(
        &self,
        directory: &Directory<K>,
        selected: &[usize],
    ) -> Option<Vec<LeafNode<K, V>>> {
        let mut pending = Vec::with_capacity(selected.len());
        for index in selected {
            pending.push(self.load_leaf(directory.leaves[*index].leaf_id));
        }
        let fetched = futures::future::join_all(pending).await;

        let mut leaves = Vec::with_capacity(fetched.len());
        for (position, leaf) in fetched.into_iter().enumerate() {
            let leaf = leaf?;
            let index = selected[position];
            let expected_low = directory.leaves[index].low.as_ref();
            let expected_high = match directory.leaves.get(index + 1) {
                Some(next) => next.low.as_ref(),
                None => None,
            };
            if leaf.low.as_ref() != expected_low || leaf.high.as_ref() != expected_high {
                return None;
            }
            leaves.push(leaf);
        }
        return Some(leaves);
    }

    async fn paginate(&self, count: usize, offset: usize, descending: bool) -> Vec<(K, V)> {
        if count == 0 {
            return Vec::new();
        }

        for _ in 0..MAX_STALE_RETRIES {
            let directory = self.load_directory().await;
            let plan = plan_pagination(&directory, count, offset, descending);
            if plan.selected.is_empty() {
                return Vec::new();
            }

            let Some(leaves) = self.fetch_leaves(&directory, &plan.selected).await else {
                continue;
            };

            let mut results = Vec::with_capacity(count);
            let mut skip = plan.skip_in_first_leaf;
            for leaf in leaves {
                if descending {
                    for entry in leaf.entries.into_iter().rev().skip(skip) {
                        results.push((entry.key, entry.value));
                        if results.len() >= count {
                            return results;
                        }
                    }
                } else {
                    for entry in leaf.entries.into_iter().skip(skip) {
                        results.push((entry.key, entry.value));
                        if results.len() >= count {
                            return results;
                        }
                    }
                }
                skip = 0;
            }
            return results;
        }
        return Vec::new();
    }
}

struct PaginationPlan {
    selected: Vec<usize>,
    skip_in_first_leaf: usize,
}

#[derive(BorshDeserialize)]
struct Directory<K> {
    #[allow(dead_code)]
    next_leaf_id: u64,
    len: u64,
    leaves: Vec<LeafRef<K>>,
}

#[derive(BorshDeserialize, Clone)]
struct LeafRef<K> {
    low: Option<K>,
    leaf_id: u64,
    count: u64,
}

#[derive(BorshDeserialize, Clone)]
struct LeafNode<K, V> {
    low: Option<K>,
    high: Option<K>,
    entries: Vec<LeafEntry<K, V>>,
}

#[derive(BorshDeserialize, Clone)]
struct LeafEntry<K, V> {
    key: K,
    value: V,
}

fn locate<K: Ord>(directory: &Directory<K>, key: &K) -> Option<usize> {
    if directory.leaves.is_empty() {
        return None;
    }
    let following = directory.leaves.partition_point(|leaf_ref| match &leaf_ref.low {
        Some(low) => low <= key,
        None => true,
    });
    return Some(following.saturating_sub(1));
}

fn leaf_owns<K: Ord, V>(leaf: &LeafNode<K, V>, key: &K) -> bool {
    if let Some(low) = &leaf.low {
        if key < low {
            return false;
        }
    }
    if let Some(high) = &leaf.high {
        if key >= high {
            return false;
        }
    }
    return true;
}

fn plan_pagination<K>(
    directory: &Directory<K>,
    count: usize,
    offset: usize,
    descending: bool,
) -> PaginationPlan {
    let mut plan = PaginationPlan { selected: Vec::new(), skip_in_first_leaf: 0 };
    let mut skip = offset as u64;
    let mut wanted = count;

    let mut order: Vec<usize> = (0..directory.leaves.len()).collect();
    if descending {
        order.reverse();
    }

    for index in order {
        if wanted == 0 {
            break;
        }
        let leaf_count = directory.leaves[index].count;
        if skip >= leaf_count {
            skip -= leaf_count;
            continue;
        }
        if plan.selected.is_empty() {
            plan.skip_in_first_leaf = skip as usize;
        }
        let available = (leaf_count - skip) as usize;
        wanted = wanted.saturating_sub(available);
        skip = 0;
        plan.selected.push(index);
    }
    return plan;
}

use borsh::BorshDeserialize;
use std::fmt;
use std::io;
use std::marker::PhantomData;
use std::sync::Arc;
use vastrum_rpc_client::{RpcClient, RpcProvider};
