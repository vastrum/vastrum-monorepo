use crate::runtime;
use borsh::{BorshDeserialize, BorshSerialize};
use std::marker::PhantomData;

const LEAF_CAP: usize = 256;
const MERGE_THRESHOLD: usize = LEAF_CAP / 2;

/// KvBTree, similar semantics to rust BTree, provides sorted keys
#[derive(BorshSerialize, BorshDeserialize, Clone)]
pub struct KvBTree<K, V> {
    nonce: u64,
    #[borsh(skip)]
    _phantom: PhantomData<(K, V)>,
}

impl<K, V> Default for KvBTree<K, V> {
    fn default() -> Self {
        Self { nonce: runtime::next_nonce(), _phantom: PhantomData }
    }
}

impl<K, V> KvBTree<K, V>
where
    K: Ord + Clone + BorshSerialize + BorshDeserialize,
    V: Clone + BorshSerialize + BorshDeserialize,
{
    pub fn new() -> Self {
        return Self::default();
    }

    pub fn length(&self) -> u64 {
        return self.load_directory().len;
    }

    pub fn is_empty(&self) -> bool {
        return self.load_directory().leaves.is_empty();
    }

    pub fn get(&self, key: &K) -> Option<V> {
        let directory = self.load_directory();
        let index = locate(&directory, key)?;
        let leaf = self.load_leaf(directory.leaves[index].leaf_id);
        let entry_index = leaf.entries.binary_search_by(|entry| entry.key.cmp(key)).ok()?;
        return Some(leaf.entries[entry_index].value.clone());
    }

    /// Get the key in the btree with smallest key, ie first in ordered list
    pub fn first(&self) -> Option<(K, V)> {
        let directory = self.load_directory();
        let leaf_ref = directory.leaves.first()?;
        let leaf = self.load_leaf(leaf_ref.leaf_id);
        let entry = leaf.entries.into_iter().next()?;
        return Some((entry.key, entry.value));
    }

    /// Get the key in the btree with the largest key, ie last in ordered list
    pub fn last(&self) -> Option<(K, V)> {
        let directory = self.load_directory();
        let leaf_ref = directory.leaves.last()?;
        let leaf = self.load_leaf(leaf_ref.leaf_id);
        let entry = leaf.entries.into_iter().next_back()?;
        return Some((entry.key, entry.value));
    }

    /// Get entries with key value with range of start<->end
    pub fn range(&self, start: &K, end: &K) -> Vec<(K, V)> {
        let directory = self.load_directory();
        let mut results = Vec::new();
        let Some(first_index) = locate(&directory, start) else { return results };

        for leaf_ref in &directory.leaves[first_index..] {
            let leaf_starts_past_the_range = match &leaf_ref.low {
                Some(low) => low >= end,
                None => false,
            };
            if leaf_starts_past_the_range {
                return results;
            }
            let leaf = self.load_leaf(leaf_ref.leaf_id);
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

    pub fn get_ascending_entries(&self, count: usize, offset: usize) -> Vec<(K, V)> {
        let directory = self.load_directory();
        let mut results = Vec::with_capacity(count);
        let mut skip = offset as u64;

        for leaf_ref in directory.leaves.iter() {
            if results.len() >= count {
                break;
            }
            if skip >= leaf_ref.count {
                skip -= leaf_ref.count;
                continue;
            }
            let leaf = self.load_leaf(leaf_ref.leaf_id);
            for entry in leaf.entries.into_iter().skip(skip as usize) {
                results.push((entry.key, entry.value));
                if results.len() >= count {
                    break;
                }
            }
            skip = 0;
        }
        return results;
    }

    pub fn get_descending_entries(&self, count: usize, offset: usize) -> Vec<(K, V)> {
        let directory = self.load_directory();
        let mut results = Vec::with_capacity(count);
        let mut skip = offset as u64;

        for leaf_ref in directory.leaves.iter().rev() {
            if results.len() >= count {
                break;
            }
            if skip >= leaf_ref.count {
                skip -= leaf_ref.count;
                continue;
            }
            let leaf = self.load_leaf(leaf_ref.leaf_id);
            for entry in leaf.entries.into_iter().rev().skip(skip as usize) {
                results.push((entry.key, entry.value));
                if results.len() >= count {
                    break;
                }
            }
            skip = 0;
        }
        return results;
    }

    pub fn insert(&self, key: K, value: V) {
        let mut directory = self.load_directory();

        let Some(index) = locate(&directory, &key) else {
            self.create_first_leaf(&mut directory, key, value);
            self.store_directory(&directory);
            return;
        };

        let leaf_id = directory.leaves[index].leaf_id;
        let mut leaf = self.load_leaf(leaf_id);

        match leaf.entries.binary_search_by(|entry| entry.key.cmp(&key)) {
            Ok(entry_index) => {
                leaf.entries[entry_index].value = value;
                self.store_leaf(leaf_id, &leaf);
            }
            Err(entry_index) => {
                leaf.entries.insert(entry_index, LeafEntry { key, value });
                directory.len += 1;
                directory.leaves[index].count += 1;

                if leaf.entries.len() > LEAF_CAP {
                    self.split_leaf(&mut directory, index, leaf_id, leaf);
                } else {
                    self.store_leaf(leaf_id, &leaf);
                }
                self.store_directory(&directory);
            }
        }
    }

    pub fn remove(&self, key: &K) -> bool {
        let mut directory = self.load_directory();
        let Some(index) = locate(&directory, key) else { return false };

        let leaf_id = directory.leaves[index].leaf_id;
        let mut leaf = self.load_leaf(leaf_id);
        let Ok(entry_index) = leaf.entries.binary_search_by(|entry| entry.key.cmp(key)) else {
            return false;
        };

        leaf.entries.remove(entry_index);
        directory.len -= 1;
        directory.leaves[index].count -= 1;

        if leaf.entries.is_empty() {
            self.delete_leaf(leaf_id);
            directory.leaves.remove(index);
            self.reopen_lower_bound(&mut directory);
        } else {
            self.store_leaf(leaf_id, &leaf);
            self.merge_if_underfull(&mut directory, index);
        }

        self.store_directory(&directory);
        return true;
    }
}

impl<K, V> KvBTree<K, V>
where
    K: Ord + Clone + BorshSerialize + BorshDeserialize,
    V: Clone + BorshSerialize + BorshDeserialize,
{
    fn directory_key(&self) -> String {
        return format!("n.{}.meta", self.nonce);
    }

    fn leaf_key(&self, leaf_id: u64) -> String {
        return format!("n.{}.leaf.{}", self.nonce, leaf_id);
    }

    fn load_directory(&self) -> Directory<K> {
        let bytes = runtime::kv_get(&self.directory_key());
        if bytes.is_empty() {
            return Directory { next_leaf_id: 0, len: 0, leaves: Vec::new() };
        }
        return borsh::from_slice(&bytes).unwrap();
    }

    fn store_directory(&self, directory: &Directory<K>) {
        runtime::kv_insert(&self.directory_key(), &borsh::to_vec(directory).unwrap());
    }

    fn load_leaf(&self, leaf_id: u64) -> LeafNode<K, V> {
        let bytes = runtime::kv_get(&self.leaf_key(leaf_id));
        return borsh::from_slice(&bytes).unwrap();
    }

    fn store_leaf(&self, leaf_id: u64, leaf: &LeafNode<K, V>) {
        runtime::kv_insert(&self.leaf_key(leaf_id), &borsh::to_vec(leaf).unwrap());
    }

    fn delete_leaf(&self, leaf_id: u64) {
        runtime::kv_delete(&self.leaf_key(leaf_id));
    }

    fn create_first_leaf(&self, directory: &mut Directory<K>, key: K, value: V) {
        let leaf_id = directory.next_leaf_id;
        directory.next_leaf_id += 1;

        let leaf = LeafNode { low: None, high: None, entries: vec![LeafEntry { key, value }] };
        self.store_leaf(leaf_id, &leaf);

        directory.leaves.push(LeafRef { low: None, leaf_id, count: 1 });
        directory.len = 1;
    }

    fn split_leaf(
        &self,
        directory: &mut Directory<K>,
        index: usize,
        leaf_id: u64,
        leaf: LeafNode<K, V>,
    ) {
        let mut leaf = leaf;
        let midpoint = leaf.entries.len() / 2;
        let right_entries = leaf.entries.split_off(midpoint);
        let split_key = right_entries[0].key.clone();

        let right_id = directory.next_leaf_id;
        directory.next_leaf_id += 1;
        let right_count = right_entries.len() as u64;
        let right = LeafNode {
            low: Some(split_key.clone()),
            high: leaf.high.clone(),
            entries: right_entries,
        };
        self.store_leaf(right_id, &right);

        let left_count = leaf.entries.len() as u64;
        leaf.high = Some(split_key.clone());
        self.store_leaf(leaf_id, &leaf);

        directory.leaves[index].count = left_count;
        directory.leaves.insert(
            index + 1,
            LeafRef { low: Some(split_key), leaf_id: right_id, count: right_count },
        );
    }

    fn merge_if_underfull(&self, directory: &mut Directory<K>, index: usize) {
        if directory.leaves[index].count as usize >= MERGE_THRESHOLD {
            return;
        }

        let left_index = if index > 0 { index - 1 } else { index };
        let right_index = left_index + 1;
        if right_index >= directory.leaves.len() {
            return;
        }

        let combined = directory.leaves[left_index].count + directory.leaves[right_index].count;
        if combined as usize > LEAF_CAP {
            return;
        }

        let left_id = directory.leaves[left_index].leaf_id;
        let right_id = directory.leaves[right_index].leaf_id;
        let mut left = self.load_leaf(left_id);
        let right = self.load_leaf(right_id);

        left.entries.extend(right.entries);
        left.high = right.high;
        self.store_leaf(left_id, &left);
        self.delete_leaf(right_id);

        directory.leaves.remove(right_index);
        directory.leaves[left_index].count = combined;
    }

    fn reopen_lower_bound(&self, directory: &mut Directory<K>) {
        let Some(first) = directory.leaves.first_mut() else { return };
        if first.low.is_none() {
            return;
        }
        first.low = None;

        let leaf_id = first.leaf_id;
        let mut leaf = self.load_leaf(leaf_id);
        leaf.low = None;
        self.store_leaf(leaf_id, &leaf);
    }
}

#[derive(BorshSerialize, BorshDeserialize)]
struct Directory<K> {
    next_leaf_id: u64,
    len: u64,
    leaves: Vec<LeafRef<K>>,
}

#[derive(BorshSerialize, BorshDeserialize, Clone)]
struct LeafRef<K> {
    low: Option<K>,
    leaf_id: u64,
    count: u64,
}

#[derive(BorshSerialize, BorshDeserialize, Clone)]
struct LeafNode<K, V> {
    low: Option<K>,
    high: Option<K>,
    entries: Vec<LeafEntry<K, V>>,
}

#[derive(BorshSerialize, BorshDeserialize, Clone)]
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
