use crate::KvBTree;
use crate::KvVec;
use borsh::{BorshDeserialize, BorshSerialize};

#[derive(BorshSerialize, BorshDeserialize, Clone)]
struct SortedEntry<S, V> {
    sort_key: S,
    value: V,
}

#[derive(BorshSerialize, BorshDeserialize, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct IndexKey<S> {
    sort_key: S,
    id: u64,
}

#[derive(BorshSerialize, BorshDeserialize, Clone)]
pub struct KvVecBTree<S, V> {
    vec: KvVec<SortedEntry<S, V>>,
    index: KvBTree<IndexKey<S>, u64>,
}

impl<S, V> Default for KvVecBTree<S, V>
where
    S: Ord + Clone + BorshSerialize + BorshDeserialize,
    V: BorshSerialize + BorshDeserialize,
{
    fn default() -> Self {
        Self { vec: KvVec::new(), index: KvBTree::new() }
    }
}

impl<S, V> KvVecBTree<S, V>
where
    S: Ord + Clone + BorshSerialize + BorshDeserialize,
    V: BorshSerialize + BorshDeserialize,
{
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&self, sort_key: S, value: V) -> u64 {
        let entry = SortedEntry { sort_key: sort_key.clone(), value };
        let id = self.vec.push(entry);
        self.index.insert(IndexKey { sort_key, id }, id);
        return id;
    }

    pub fn get(&self, id: u64) -> Option<V> {
        let entry = self.vec.get(id)?;
        return Some(entry.value);
    }

    pub fn update(&self, id: u64, new_sort_key: S, value: V) {
        let old = self.vec.get(id).expect("update: id not found");
        self.index.remove(&IndexKey { sort_key: old.sort_key, id });
        let entry = SortedEntry { sort_key: new_sort_key.clone(), value };
        self.vec.set(id, entry);
        self.index.insert(IndexKey { sort_key: new_sort_key, id }, id);
    }

    pub fn remove(&self, id: u64) {
        let Some(old) = self.vec.get(id) else { return };
        self.index.remove(&IndexKey { sort_key: old.sort_key, id });
        self.vec.delete(id);
    }

    pub fn next_id(&self) -> u64 {
        let next_id = self.vec.length();
        return next_id;
    }

    pub fn range(&self, start: &S, end: &S) -> Vec<V> {
        let from = IndexKey { sort_key: start.clone(), id: 0 };
        let to = IndexKey { sort_key: end.clone(), id: 0 };
        let entries = self.index.range(&from, &to);
        let mut results = Vec::new();
        for (_, id) in entries {
            let entry = self.vec.get(id).unwrap();
            results.push(entry.value);
        }
        return results;
    }

    pub fn get_descending_entries(&self, count: usize, offset: usize) -> Vec<V> {
        let entries = self.index.get_descending_entries(count, offset);
        let mut results = Vec::new();
        for (_, id) in entries {
            let entry = self.vec.get(id).unwrap();
            results.push(entry.value);
        }
        return results;
    }

    pub fn get_ascending_entries(&self, count: usize, offset: usize) -> Vec<V> {
        let entries = self.index.get_ascending_entries(count, offset);
        let mut results = Vec::new();
        for (_, id) in entries {
            let entry = self.vec.get(id).unwrap();
            results.push(entry.value);
        }
        return results;
    }

    pub fn length(&self) -> u64 {
        let length = self.index.length();
        return length;
    }

    pub fn is_empty(&self) -> bool {
        return self.index.is_empty();
    }
}
