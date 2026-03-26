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

    async fn load_inner(&self, height: u64) -> KvBTreeInner<K, V> {
        return KvBTreeInner::load(self.nonce, self.client.clone(), height).await;
    }

    pub async fn get_height(&self) -> u64 {
        return self.client.get_latest_block_height().await.unwrap_or(0);
    }

    pub async fn length(&self) -> u64 {
        let locked_height = self.get_height().await;
        let inner = self.load_inner(locked_height).await;
        return inner.len;
    }

    pub async fn is_empty(&self) -> bool {
        let locked_height = self.get_height().await;
        let inner = self.load_inner(locked_height).await;
        return inner.root_node_id.is_none();
    }

    pub async fn get(&self, key: &K) -> Option<V> {
        let locked_height = self.get_height().await;
        let inner = self.load_inner(locked_height).await;
        return inner.get(key).await;
    }

    pub async fn first(&self) -> Option<(K, V)> {
        let locked_height = self.get_height().await;
        let inner = self.load_inner(locked_height).await;
        return inner.first().await;
    }

    pub async fn last(&self) -> Option<(K, V)> {
        let locked_height = self.get_height().await;
        let inner = self.load_inner(locked_height).await;
        return inner.last().await;
    }

    pub async fn range(&self, start: &K, end: &K) -> Vec<(K, V)> {
        let locked_height = self.get_height().await;
        let inner = self.load_inner(locked_height).await;
        return inner.range(start, end).await;
    }

    pub async fn get_ascending_entries(&self, count: usize, offset: usize) -> Vec<(K, V)> {
        let locked_height = self.get_height().await;
        let inner = self.load_inner(locked_height).await;
        return inner.get_ascending_entries(count, offset).await;
    }

    pub async fn get_descending_entries(&self, count: usize, offset: usize) -> Vec<(K, V)> {
        let locked_height = self.get_height().await;
        let inner = self.load_inner(locked_height).await;
        return inner.get_descending_entries(count, offset).await;
    }

    /*
       pub async fn get_at(&self, key: &K, height: u64) -> Option<V> {
           self.load_inner(height).await.get(key).await
       }

       pub async fn first_at(&self, height: u64) -> Option<(K, V)> {
           self.load_inner(height).await.first().await
       }

       pub async fn last_at(&self, height: u64) -> Option<(K, V)> {
           self.load_inner(height).await.last().await
       }
    */
    pub async fn range_at(&self, start: &K, end: &K, height: u64) -> Vec<(K, V)> {
        let inner = self.load_inner(height).await;
        return inner.range(start, end).await;
    }

    pub async fn get_ascending_entries_at(
        &self,
        count: usize,
        offset: usize,
        height: u64,
    ) -> Vec<(K, V)> {
        let inner = self.load_inner(height).await;
        return inner.get_ascending_entries(count, offset).await;
    }

    pub async fn get_descending_entries_at(
        &self,
        count: usize,
        offset: usize,
        height: u64,
    ) -> Vec<(K, V)> {
        let inner = self.load_inner(height).await;
        return inner.get_descending_entries(count, offset).await;
    }
}

struct KvBTreeInner<K, V> {
    nonce: u64,
    root_node_id: Option<u64>,
    len: u64,
    locked_height: u64,
    client: Arc<RpcClient>,
    _phantom: PhantomData<(K, V)>,
}

impl<K, V> KvBTreeInner<K, V>
where
    K: Ord + Clone + BorshDeserialize,
    V: Clone + BorshDeserialize,
{
    async fn load(nonce: u64, client: Arc<RpcClient>, locked_height: u64) -> Self {
        let key = Self::meta_key(nonce);
        let (root, len) = match client.get_key_value_at_height(key, locked_height).await {
            Some(bytes) if !bytes.is_empty() => {
                let mut reader = &bytes[..];
                let _nonce = u64::deserialize_reader(&mut reader).unwrap();
                let root_node_id = Option::<u64>::deserialize_reader(&mut reader).unwrap();
                let _next_id = u64::deserialize_reader(&mut reader).unwrap();
                let len = u64::deserialize_reader(&mut reader).unwrap();
                (root_node_id, len)
            }
            _ => (None, 0),
        };
        return Self {
            nonce,
            root_node_id: root,
            len,
            locked_height,
            client,
            _phantom: PhantomData,
        };
    }

    fn meta_key(nonce: u64) -> String {
        return format!("n.{}.meta", nonce);
    }

    fn node_key(&self, id: u64) -> String {
        return format!("n.{}.node.{}", self.nonce, id);
    }

    async fn get_node(&self, id: u64) -> Option<Node<K, V>> {
        let key = self.node_key(id);
        let bytes = self.client.get_key_value_at_height(key, self.locked_height).await?;
        if bytes.is_empty() {
            return None;
        } else {
            return crate::with_deser_client(&self.client, || borsh::from_slice(&bytes).ok());
        }
    }

    async fn get_leaf(&self, id: u64) -> LeafNode<K, V> {
        let Node::Leaf(leaf) = self.get_node(id).await.unwrap() else { unreachable!() };
        return leaf;
    }

    async fn find_leaf(&self, key: &K) -> Option<LeafNode<K, V>> {
        let mut current_id = self.root_node_id?;

        loop {
            match self.get_node(current_id).await.unwrap() {
                Node::Leaf(leaf) => return Some(leaf),
                Node::Internal(internal) => {
                    let mut child_idx = 0;
                    for node_key in &internal.keys {
                        if key < node_key {
                            break;
                        }
                        child_idx += 1;
                    }
                    current_id = internal.children[child_idx];
                }
            }
        }
    }

    async fn descend_to_position(&self, node: Node<K, V>, pos: u64) -> (LeafNode<K, V>, usize) {
        let mut current_node = node;
        let mut position_in_subtree = pos;
        loop {
            match current_node {
                Node::Leaf(leaf) => return (leaf, position_in_subtree as usize),
                Node::Internal(ref internal) => {
                    let mut i = 0;
                    while position_in_subtree >= internal.counts[i] {
                        position_in_subtree -= internal.counts[i];
                        i += 1;
                    }
                    current_node = self.get_node(internal.children[i]).await.unwrap();
                }
            }
        }
    }

    async fn get(&self, key: &K) -> Option<V> {
        let leaf = self.find_leaf(key).await?;
        let idx = leaf.keys.binary_search(key).ok()?;
        return Some(leaf.values[idx].clone());
    }

    async fn first(&self) -> Option<(K, V)> {
        let mut current_id = self.root_node_id?;

        loop {
            match self.get_node(current_id).await.unwrap() {
                Node::Leaf(leaf) => {
                    return Some((leaf.keys[0].clone(), leaf.values[0].clone()));
                }
                Node::Internal(node) => {
                    current_id = node.children[0];
                }
            }
        }
    }

    async fn last(&self) -> Option<(K, V)> {
        let mut current_id = self.root_node_id?;

        loop {
            match self.get_node(current_id).await.unwrap() {
                Node::Leaf(leaf) => {
                    let i = leaf.keys.len() - 1;
                    return Some((leaf.keys[i].clone(), leaf.values[i].clone()));
                }
                Node::Internal(node) => {
                    current_id = *node.children.last().unwrap();
                }
            }
        }
    }

    async fn range(&self, start: &K, end: &K) -> Vec<(K, V)> {
        let mut results = Vec::new();

        let Some(mut leaf) = self.find_leaf(start).await else {
            return results;
        };

        loop {
            for (k, v) in leaf.keys.iter().zip(leaf.values.iter()) {
                if k >= end {
                    return results;
                }
                if k >= start {
                    results.push((k.clone(), v.clone()));
                }
            }

            let Some(next_id) = leaf.next else { break };
            leaf = self.get_leaf(next_id).await;
        }

        return results;
    }

    async fn get_ascending_entries(&self, count: usize, offset: usize) -> Vec<(K, V)> {
        if count == 0 {
            return Vec::new();
        }

        let Some(root_id) = self.root_node_id else {
            return Vec::new();
        };
        let root_node = self.get_node(root_id).await.unwrap();

        if let Node::Leaf(ref leaf) = root_node {
            let mut results = Vec::new();
            for (k, v) in leaf.keys.iter().zip(leaf.values.iter()).skip(offset).take(count) {
                results.push((k.clone(), v.clone()));
            }
            return results;
        }

        let Node::Internal(ref root) = root_node else { unreachable!() };
        let total_entries: u64 = root.counts.iter().sum();

        if offset as u64 >= total_entries {
            return Vec::new();
        }

        let target = offset as u64;
        let (leaf, pos_in_leaf) = self.descend_to_position(root_node, target).await;

        let mut results = Vec::new();

        for i in pos_in_leaf..leaf.keys.len() {
            results.push((leaf.keys[i].clone(), leaf.values[i].clone()));
            if results.len() >= count {
                return results;
            }
        }

        let mut next_id = leaf.next;
        while let Some(id) = next_id {
            let leaf = self.get_leaf(id).await;
            for (k, v) in leaf.keys.into_iter().zip(leaf.values.into_iter()) {
                results.push((k, v));
                if results.len() >= count {
                    return results;
                }
            }
            next_id = leaf.next;
        }

        return results;
    }

    async fn get_descending_entries(&self, count: usize, offset: usize) -> Vec<(K, V)> {
        if count == 0 {
            return Vec::new();
        }

        let Some(root_id) = self.root_node_id else {
            return Vec::new();
        };
        let root_node = self.get_node(root_id).await.unwrap();

        if let Node::Leaf(ref leaf) = root_node {
            let mut results = Vec::new();
            for (k, v) in leaf.keys.iter().zip(leaf.values.iter()).rev().skip(offset).take(count) {
                results.push((k.clone(), v.clone()));
            }
            return results;
        }

        let Node::Internal(ref root) = root_node else { unreachable!() };
        let total_entries: u64 = root.counts.iter().sum();

        if offset as u64 >= total_entries {
            return Vec::new();
        }

        let target = total_entries - offset as u64 - 1;
        let (leaf, pos_in_leaf) = self.descend_to_position(root_node, target).await;

        let mut results = Vec::new();

        for i in (0..=pos_in_leaf).rev() {
            results.push((leaf.keys[i].clone(), leaf.values[i].clone()));
            if results.len() >= count {
                return results;
            }
        }

        let mut prev_id = leaf.prev;
        while let Some(id) = prev_id {
            let leaf = self.get_leaf(id).await;
            for (k, v) in leaf.keys.into_iter().zip(leaf.values.into_iter()).rev() {
                results.push((k, v));
                if results.len() >= count {
                    return results;
                }
            }
            prev_id = leaf.prev;
        }

        return results;
    }
}

#[derive(Clone, BorshDeserialize)]
struct InternalNode<K> {
    keys: Vec<K>,
    children: Vec<u64>,
    counts: Vec<u64>,
}

#[derive(Clone, BorshDeserialize)]
struct LeafNode<K, V> {
    keys: Vec<K>,
    values: Vec<V>,
    prev: Option<u64>,
    next: Option<u64>,
}

#[derive(Clone, BorshDeserialize)]
enum Node<K, V> {
    Internal(InternalNode<K>),
    Leaf(LeafNode<K, V>),
}
use borsh::BorshDeserialize;
use vastrum_rpc_client::{RpcClient, RpcProvider};
use std::fmt;
use std::io;
use std::marker::PhantomData;
use std::sync::Arc;
