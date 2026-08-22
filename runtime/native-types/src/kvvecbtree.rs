use borsh::{BorshDeserialize, BorshSerialize};
use std::fmt;
use std::io;
use std::sync::Arc;
use vastrum_rpc_client::RpcClient;

use crate::{KvBTree, KvVec};

#[derive(BorshSerialize, BorshDeserialize, Clone, Debug)]
struct SortedEntry<S, V> {
    sort_key: S,
    value: V,
}

#[derive(BorshSerialize, BorshDeserialize, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct IndexKey<S> {
    sort_key: S,
    id: u64,
}

pub struct KvVecBTree<S, V> {
    vec: KvVec<SortedEntry<S, V>>,
    index: KvBTree<IndexKey<S>, u64>,
}

impl<S, V> Clone for KvVecBTree<S, V> {
    fn clone(&self) -> Self {
        Self { vec: self.vec.clone(), index: self.index.clone() }
    }
}

impl<S, V> fmt::Debug for KvVecBTree<S, V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("KvVecBTree").field("vec", &self.vec).field("index", &self.index).finish()
    }
}

impl<S, V> BorshDeserialize for KvVecBTree<S, V> {
    fn deserialize_reader<R: io::Read>(reader: &mut R) -> io::Result<Self> {
        //to pass rpc client to child kv data types as they need it to get()
        //very complicated but works
        let vec = KvVec::deserialize_reader(reader)?;
        let index = KvBTree::deserialize_reader(reader)?;
        Ok(Self { vec, index })
    }
}

impl<S, V> KvVecBTree<S, V>
where
    S: Ord + Clone + BorshSerialize + BorshDeserialize,
    V: BorshDeserialize,
{
    pub fn new(vec_nonce: u64, index_nonce: u64, client: Arc<RpcClient>) -> Self {
        Self {
            vec: KvVec::new(vec_nonce, client.clone()),
            index: KvBTree::new(index_nonce, client),
        }
    }

    pub async fn get(&self, id: u64) -> Option<V> {
        let Some(entry) = self.vec.get(id).await else { return None };
        return Some(entry.value);
    }

    pub async fn range(&self, start: &S, end: &S) -> Vec<V> {
        let from = IndexKey { sort_key: start.clone(), id: 0 };
        let to = IndexKey { sort_key: end.clone(), id: 0 };
        let entries = self.index.range(&from, &to).await;
        return self.load_values(&entries).await;
    }

    pub async fn get_descending_entries(&self, count: usize, offset: usize) -> Vec<V> {
        let entries = self.index.get_descending_entries(count, offset).await;
        return self.load_values(&entries).await;
    }

    pub async fn get_ascending_entries(&self, count: usize, offset: usize) -> Vec<V> {
        let entries = self.index.get_ascending_entries(count, offset).await;
        return self.load_values(&entries).await;
    }

    async fn load_values(&self, entries: &[(IndexKey<S>, u64)]) -> Vec<V> {
        let mut pending = Vec::with_capacity(entries.len());
        for (_, id) in entries {
            pending.push(self.vec.get(*id));
        }
        let fetched = futures::future::join_all(pending).await;

        let mut results = Vec::new();
        for entry in fetched {
            if let Some(entry) = entry {
                results.push(entry.value);
            }
        }
        return results;
    }

    pub async fn length(&self) -> u64 {
        self.index.length().await
    }

    pub async fn is_empty(&self) -> bool {
        self.index.is_empty().await
    }
}
