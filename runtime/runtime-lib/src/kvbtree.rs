const MAX_KEYS: usize = 64;
const MIN_KEYS: usize = MAX_KEYS / 2;

// kvbtree, split into two structs in order to a avoid sync issues
// ie if len/root_node_id lived in kvbtree then values get stale if have two concurrent references
// to same kvbtree, can see kvbtree as a pointer to the real storage which kvbtree inner handles
// kvbtree just loads the data

///KvBtree, similar semantics to rust Btree, provides sorted keys
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

    fn load_inner(&self) -> KvBTreeInner<K, V> {
        KvBTreeInner::load(self.nonce)
    }

    pub fn length(&self) -> u64 {
        let inner = self.load_inner();
        return inner.len;
    }

    pub fn is_empty(&self) -> bool {
        let inner = self.load_inner();
        return inner.root_node_id.is_none();
    }

    pub fn get(&self, key: &K) -> Option<V> {
        let inner = self.load_inner();
        return inner.get(key);
    }

    /// Get the key in the btree with smallest key, ie first in ordered list
    pub fn first(&self) -> Option<(K, V)> {
        let inner = self.load_inner();
        return inner.first();
    }

    /// Get the key in the btree with the largest key, ie last in ordered list
    pub fn last(&self) -> Option<(K, V)> {
        let inner = self.load_inner();
        return inner.last();
    }

    /// Get entries with key value with range of start<->end
    pub fn range(&self, start: &K, end: &K) -> Vec<(K, V)> {
        let inner = self.load_inner();
        return inner.range(start, end);
    }

    pub fn get_ascending_entries(&self, count: usize, offset: usize) -> Vec<(K, V)> {
        let inner = self.load_inner();
        return inner.get_ascending_entries(count, offset);
    }

    pub fn get_descending_entries(&self, count: usize, offset: usize) -> Vec<(K, V)> {
        let inner = self.load_inner();
        return inner.get_descending_entries(count, offset);
    }

    pub fn insert(&self, key: K, value: V) {
        let mut inner = self.load_inner();
        inner.insert(key, value);
        inner.save();
    }

    /// Remove a key, returns true if key was removed
    pub fn remove(&self, key: &K) -> bool {
        let mut inner = self.load_inner();
        let value_was_removed = inner.remove(key);
        inner.save();
        return value_was_removed;
    }
}

#[derive(BorshSerialize, BorshDeserialize)]
struct KvBTreeInner<K, V> {
    nonce: u64,
    root_node_id: Option<u64>,
    next_id: u64,
    len: u64,
    #[borsh(skip)]
    _phantom: PhantomData<(K, V)>,
}

impl<K, V> KvBTreeInner<K, V> {
    fn load(nonce: u64) -> Self {
        let key = format!("n.{}.meta", nonce);
        let bytes = runtime::kv_get(&key);
        if bytes.is_empty() {
            return KvBTreeInner {
                nonce,
                root_node_id: None,
                next_id: 0,
                len: 0,
                _phantom: PhantomData,
            };
        } else {
            return borsh::from_slice(&bytes).unwrap();
        }
    }

    fn save(&self) {
        let key = format!("n.{}.meta", self.nonce);
        runtime::kv_insert(&key, &borsh::to_vec(self).unwrap());
    }
}

impl<K, V> KvBTreeInner<K, V>
where
    K: Ord + Clone + BorshSerialize + BorshDeserialize,
    V: Clone + BorshSerialize + BorshDeserialize,
{
    fn node_key(&self, id: u64) -> String {
        format!("n.{}.node.{}", self.nonce, id)
    }

    fn get_node(&self, id: u64) -> Node<K, V> {
        let bytes = runtime::kv_get(&self.node_key(id));
        let node = borsh::from_slice(&bytes).unwrap();
        return node;
    }

    fn set_node(&self, id: u64, node: &Node<K, V>) {
        let key = self.node_key(id);
        let value = borsh::to_vec(node).unwrap();
        runtime::kv_insert(&key, &value);
    }

    fn delete_node(&self, id: u64) {
        let key = self.node_key(id);
        runtime::kv_delete(&key);
    }

    fn get_leaf_node(&self, id: u64) -> LeafNode<K, V> {
        let Node::Leaf(leaf) = self.get_node(id) else { unreachable!() };
        return leaf;
    }

    fn get_internal_node(&self, id: u64) -> InternalNode<K> {
        let Node::Internal(node) = self.get_node(id) else { unreachable!() };
        return node;
    }

    fn set_leaf_node(&self, id: u64, leaf: LeafNode<K, V>) {
        self.set_node(id, &Node::Leaf(leaf));
    }

    fn set_internal_node(&self, id: u64, node: InternalNode<K>) {
        self.set_node(id, &Node::Internal(node));
    }

    fn find_path_and_leaf(&self, key: &K) -> Option<PathToLeaf<K, V>> {
        let mut path = Vec::new();
        let mut current_id = self.root_node_id?;

        loop {
            match self.get_node(current_id) {
                Node::Leaf(leaf) => return Some(PathToLeaf { path, leaf_id: current_id, leaf }),
                Node::Internal(internal) => {
                    let mut child_idx = 0;
                    for node_key in &internal.keys {
                        if key < node_key {
                            break;
                        }
                        child_idx += 1;
                    }
                    let child_id = internal.children[child_idx];
                    path.push(PathStep { node_id: current_id, child_idx, node: internal });
                    current_id = child_id;
                }
            }
        }
    }

    fn descend_to_position(&self, node: Node<K, V>, pos: u64) -> (LeafNode<K, V>, usize) {
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
                    current_node = self.get_node(internal.children[i]);
                }
            }
        }
    }

    fn get(&self, key: &K) -> Option<V> {
        let leaf = self.find_path_and_leaf(key)?.leaf;
        let idx = leaf.keys.binary_search(key).ok()?;
        let value = Some(leaf.values[idx].clone());
        return value;
    }

    fn first(&self) -> Option<(K, V)> {
        let mut current_id = self.root_node_id?;

        loop {
            match self.get_node(current_id) {
                Node::Leaf(leaf) => {
                    let key = leaf.keys[0].clone();
                    let value = leaf.values[0].clone();
                    return Some((key, value));
                }
                Node::Internal(node) => {
                    current_id = node.children[0];
                }
            }
        }
    }

    fn last(&self) -> Option<(K, V)> {
        let mut current_id = self.root_node_id?;

        loop {
            match self.get_node(current_id) {
                Node::Leaf(leaf) => {
                    let i = leaf.keys.len() - 1;
                    let key = leaf.keys[i].clone();
                    let value = leaf.values[i].clone();
                    return Some((key, value));
                }
                Node::Internal(node) => {
                    current_id = *node.children.last().unwrap();
                }
            }
        }
    }

    fn range(&self, start: &K, end: &K) -> Vec<(K, V)> {
        let mut results = Vec::new();

        let Some(PathToLeaf { mut leaf, .. }) = self.find_path_and_leaf(start) else {
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
            leaf = self.get_leaf_node(next_id);
        }

        return results;
    }

    fn get_ascending_entries(&self, count: usize, offset: usize) -> Vec<(K, V)> {
        if count == 0 {
            return Vec::new();
        }

        let root_id = match self.root_node_id {
            Some(id) => id,
            None => return Vec::new(),
        };
        let root_node = self.get_node(root_id);

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

        let (leaf, pos_in_leaf) = self.descend_to_position(root_node, target);

        let mut results = Vec::new();

        for i in pos_in_leaf..leaf.keys.len() {
            results.push((leaf.keys[i].clone(), leaf.values[i].clone()));
            if results.len() >= count {
                return results;
            }
        }

        let mut next_id = leaf.next;
        while let Some(id) = next_id {
            let leaf = self.get_leaf_node(id);

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

    fn get_descending_entries(&self, count: usize, offset: usize) -> Vec<(K, V)> {
        if count == 0 {
            return Vec::new();
        }

        let root_id = match self.root_node_id {
            Some(id) => id,
            None => return Vec::new(),
        };
        let root_node = self.get_node(root_id);

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

        let (leaf, pos_in_leaf) = self.descend_to_position(root_node, target);

        let mut results = Vec::new();

        for i in (0..=pos_in_leaf).rev() {
            results.push((leaf.keys[i].clone(), leaf.values[i].clone()));
            if results.len() >= count {
                return results;
            }
        }

        let mut prev_id = leaf.prev;
        while let Some(id) = prev_id {
            let leaf = self.get_leaf_node(id);

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

    fn increment_counts_on_path(&self, path: &mut [PathStep<K>]) {
        for step in path.iter_mut() {
            step.node.counts[step.child_idx] += 1;
            self.set_internal_node(step.node_id, step.node.clone());
        }
    }

    fn insert(&mut self, key: K, value: V) {
        let is_empty_kvbtree = self.root_node_id.is_none();

        if is_empty_kvbtree {
            let node_id = self.next_id;
            self.next_id += 1;
            let leaf = LeafNode { keys: vec![key], values: vec![value], prev: None, next: None };
            self.set_leaf_node(node_id, leaf);
            self.root_node_id = Some(node_id);
            self.len = 1;
            return;
        }

        let PathToLeaf { path, leaf_id, leaf } = self.find_path_and_leaf(&key).unwrap();

        match leaf.keys.binary_search(&key) {
            Ok(idx) => self.overwrite_leaf(leaf_id, leaf, idx, value),
            Err(idx) => self.insert_into_leaf(path, leaf_id, leaf, idx, key, value),
        }
    }

    fn overwrite_leaf(&self, leaf_id: u64, leaf: LeafNode<K, V>, idx: usize, value: V) {
        let mut leaf = leaf;
        leaf.values[idx] = value;
        self.set_leaf_node(leaf_id, leaf);
    }

    fn insert_into_leaf(
        &mut self,
        path: Vec<PathStep<K>>,
        leaf_id: u64,
        leaf: LeafNode<K, V>,
        idx: usize,
        key: K,
        value: V,
    ) {
        let mut path = path;
        let mut leaf = leaf;
        leaf.keys.insert(idx, key);
        leaf.values.insert(idx, value);
        self.len += 1;
        let need_to_split_leaf = leaf.keys.len() > MAX_KEYS;

        if need_to_split_leaf {
            let result = self.split_leaf(leaf_id, leaf);
            self.propagate_split(&mut path, result);
        } else {
            self.set_leaf_node(leaf_id, leaf);
            self.increment_counts_on_path(&mut path);
        }
    }

    fn split_leaf(&mut self, leaf_id: u64, leaf: LeafNode<K, V>) -> SplitResult<K> {
        let mid = leaf.keys.len() / 2;

        let left_count = mid as u64;
        let right_count = (leaf.keys.len() - mid) as u64;

        let left_keys = leaf.keys[..mid].to_vec();
        let left_values = leaf.values[..mid].to_vec();
        let right_keys = leaf.keys[mid..].to_vec();
        let right_values = leaf.values[mid..].to_vec();

        let separator = right_keys[0].clone();

        let right_id = self.next_id;
        self.next_id += 1;

        self.set_leaf_node(
            right_id,
            LeafNode {
                keys: right_keys,
                values: right_values,
                prev: Some(leaf_id),
                next: leaf.next,
            },
        );

        self.set_leaf_node(
            leaf_id,
            LeafNode {
                keys: left_keys,
                values: left_values,
                prev: leaf.prev,
                next: Some(right_id),
            },
        );

        if let Some(next_id) = leaf.next {
            let mut neighbor = self.get_leaf_node(next_id);
            neighbor.prev = Some(right_id);
            self.set_leaf_node(next_id, neighbor);
        }

        return SplitResult { separator, new_node_id: right_id, left_count, right_count };
    }

    fn split_internal(&mut self, node_id: u64, node: InternalNode<K>) -> SplitResult<K> {
        let mid = node.keys.len() / 2;

        let right_keys = node.keys[mid + 1..].to_vec();
        let right_children = node.children[mid + 1..].to_vec();
        let right_counts = node.counts[mid + 1..].to_vec();
        let right_total: u64 = right_counts.iter().sum();

        let right_id = self.next_id;
        self.next_id += 1;
        self.set_internal_node(
            right_id,
            InternalNode { keys: right_keys, children: right_children, counts: right_counts },
        );

        let left_keys = node.keys[..mid].to_vec();
        let left_children = node.children[..=mid].to_vec();
        let left_counts = node.counts[..=mid].to_vec();
        let left_total: u64 = left_counts.iter().sum();

        self.set_internal_node(
            node_id,
            InternalNode { keys: left_keys, children: left_children, counts: left_counts },
        );

        let separator = node.keys[mid].clone();

        return SplitResult {
            separator,
            new_node_id: right_id,
            left_count: left_total,
            right_count: right_total,
        };
    }

    fn propagate_split(&mut self, path: &mut Vec<PathStep<K>>, split: SplitResult<K>) {
        let mut current_split = split;
        loop {
            let Some(step) = path.pop() else {
                let old_root_id = self.root_node_id.unwrap();
                self.grow_root(old_root_id, current_split);
                return;
            };

            let mut parent = step.node;
            parent.insert_child(step.child_idx, &current_split);

            if parent.keys.len() > MAX_KEYS {
                current_split = self.split_internal(step.node_id, parent);
            } else {
                self.set_internal_node(step.node_id, parent);
                self.increment_counts_on_path(path);
                return;
            }
        }
    }

    fn grow_root(&mut self, left_id: u64, split: SplitResult<K>) {
        let root_id = self.next_id;
        self.next_id += 1;
        self.set_internal_node(
            root_id,
            InternalNode {
                keys: vec![split.separator],
                children: vec![left_id, split.new_node_id],
                counts: vec![split.left_count, split.right_count],
            },
        );
        self.root_node_id = Some(root_id);
    }

    fn decrement_counts_on_path(&self, path: &mut [PathStep<K>]) {
        for step in path.iter_mut() {
            step.node.counts[step.child_idx] -= 1;
            self.set_internal_node(step.node_id, step.node.clone());
        }
    }

    fn remove(&mut self, key: &K) -> bool {
        let Some(PathToLeaf { mut path, leaf_id, mut leaf }) = self.find_path_and_leaf(key) else {
            return false;
        };

        let Ok(idx) = leaf.keys.binary_search(key) else {
            return false;
        };

        leaf.keys.remove(idx);
        leaf.values.remove(idx);
        self.len -= 1;

        if leaf.keys.is_empty() && self.len == 0 {
            self.root_node_id = None;
            self.delete_node(leaf_id);
            return true;
        }

        let key_count = leaf.keys.len();
        self.set_leaf_node(leaf_id, leaf);
        self.decrement_counts_on_path(&mut path);

        if !path.is_empty() && key_count < MIN_KEYS {
            self.rebalance_leaf(path, leaf_id);
        }

        return true;
    }

    fn handle_underflow(
        &mut self,
        path: Vec<PathStep<K>>,
        parent_id: u64,
        parent: InternalNode<K>,
    ) {
        let mut path = path;
        let key_count = parent.keys.len();
        let first_child = parent.children[0];

        if path.is_empty() && key_count == 0 {
            self.root_node_id = Some(first_child);
            self.delete_node(parent_id);
        } else if !path.is_empty() && key_count < MIN_KEYS {
            self.set_internal_node(parent_id, parent);
            self.rebalance_internal(&mut path, parent_id);
        } else {
            self.set_internal_node(parent_id, parent);
        }
    }

    fn merge_leaves(
        &self,
        left_id: u64,
        left: LeafNode<K, V>,
        right_id: u64,
        right: LeafNode<K, V>,
    ) {
        let mut merged_leaf = left;
        if let Some(nn_id) = right.next {
            let mut neighbor = self.get_leaf_node(nn_id);
            neighbor.prev = Some(left_id);
            self.set_leaf_node(nn_id, neighbor);
        }
        merged_leaf.keys.extend(right.keys);
        merged_leaf.values.extend(right.values);
        merged_leaf.next = right.next;
        self.set_leaf_node(left_id, merged_leaf);
        self.delete_node(right_id);
    }

    fn merge_internals(
        &self,
        left_id: u64,
        left: InternalNode<K>,
        right_id: u64,
        right: InternalNode<K>,
        separator: K,
    ) {
        let mut merged_node = left;
        merged_node.keys.push(separator);
        merged_node.keys.extend(right.keys);
        merged_node.children.extend(right.children);
        merged_node.counts.extend(right.counts);
        self.set_internal_node(left_id, merged_node);
        self.delete_node(right_id);
    }

    fn rebalance_leaf(&mut self, path: Vec<PathStep<K>>, leaf_id: u64) {
        let mut remaining_path = path;
        let step = remaining_path.pop().unwrap();
        let parent_id = step.node_id;
        let child_idx = step.child_idx;
        let mut parent = step.node;
        let mut leaf = self.get_leaf_node(leaf_id);

        if child_idx + 1 < parent.children.len() {
            let right_id = parent.children[child_idx + 1];
            let mut right = self.get_leaf_node(right_id);
            if right.keys.len() > MIN_KEYS {
                leaf.keys.push(right.keys.remove(0));
                leaf.values.push(right.values.remove(0));
                parent.keys[child_idx] = right.keys[0].clone();
                parent.counts[child_idx] += 1;
                parent.counts[child_idx + 1] -= 1;
                self.set_leaf_node(leaf_id, leaf);
                self.set_leaf_node(right_id, right);
                self.set_internal_node(parent_id, parent);
                return;
            }
            self.merge_leaves(leaf_id, leaf, right_id, right);
            parent.absorb_child(child_idx);
            self.handle_underflow(remaining_path, parent_id, parent);
            return;
        }

        if child_idx > 0 {
            let left_id = parent.children[child_idx - 1];
            let mut left = self.get_leaf_node(left_id);
            if left.keys.len() > MIN_KEYS {
                leaf.keys.insert(0, left.keys.pop().unwrap());
                leaf.values.insert(0, left.values.pop().unwrap());
                parent.keys[child_idx - 1] = leaf.keys[0].clone();
                parent.counts[child_idx - 1] -= 1;
                parent.counts[child_idx] += 1;
                self.set_leaf_node(leaf_id, leaf);
                self.set_leaf_node(left_id, left);
                self.set_internal_node(parent_id, parent);
                return;
            }
            self.merge_leaves(left_id, left, leaf_id, leaf);
            parent.absorb_child(child_idx - 1);
            self.handle_underflow(remaining_path, parent_id, parent);
            return;
        }
    }

    fn rebalance_internal(&mut self, path: &mut Vec<PathStep<K>>, node_id: u64) {
        let step = path.pop().unwrap();
        let gp_id = step.node_id;
        let child_idx = step.child_idx;
        let mut gp = step.node;
        let mut node = self.get_internal_node(node_id);

        // Try borrow from right sibling
        if child_idx + 1 < gp.children.len() {
            let right_id = gp.children[child_idx + 1];
            let mut right = self.get_internal_node(right_id);
            if right.keys.len() > MIN_KEYS {
                node.keys.push(gp.keys[child_idx].clone());

                gp.keys[child_idx] = right.keys.remove(0);

                let moved_count = right.counts.remove(0);
                node.children.push(right.children.remove(0));
                node.counts.push(moved_count);
                gp.counts[child_idx] += moved_count;
                gp.counts[child_idx + 1] -= moved_count;
                self.set_internal_node(node_id, node);
                self.set_internal_node(right_id, right);
                self.set_internal_node(gp_id, gp);
                return;
            }
            // Borrow failed merge right into node
            self.merge_internals(node_id, node, right_id, right, gp.keys[child_idx].clone());
            gp.absorb_child(child_idx);
            self.handle_underflow(std::mem::take(path), gp_id, gp);
            return;
        }

        // Try borrow from left sibling
        if child_idx > 0 {
            let left_id = gp.children[child_idx - 1];
            let mut left = self.get_internal_node(left_id);
            if left.keys.len() > MIN_KEYS {
                node.keys.insert(0, gp.keys[child_idx - 1].clone());

                gp.keys[child_idx - 1] = left.keys.pop().unwrap();

                let moved_count = left.counts.pop().unwrap();
                node.children.insert(0, left.children.pop().unwrap());
                node.counts.insert(0, moved_count);
                gp.counts[child_idx] += moved_count;
                gp.counts[child_idx - 1] -= moved_count;
                self.set_internal_node(node_id, node);
                self.set_internal_node(left_id, left);
                self.set_internal_node(gp_id, gp);
                return;
            }
            // Borrow failed merge node into left
            self.merge_internals(left_id, left, node_id, node, gp.keys[child_idx - 1].clone());
            gp.absorb_child(child_idx - 1);
            self.handle_underflow(std::mem::take(path), gp_id, gp);
            return;
        }
    }
}

#[derive(BorshSerialize, BorshDeserialize, Clone)]
struct InternalNode<K> {
    keys: Vec<K>,
    children: Vec<u64>,
    counts: Vec<u64>,
}

#[derive(BorshSerialize, BorshDeserialize, Clone)]
struct LeafNode<K, V> {
    keys: Vec<K>,
    values: Vec<V>,
    prev: Option<u64>,
    next: Option<u64>,
}

#[derive(BorshSerialize, BorshDeserialize, Clone)]
enum Node<K, V> {
    Internal(InternalNode<K>),
    Leaf(LeafNode<K, V>),
}

struct PathStep<K> {
    node_id: u64,
    child_idx: usize,
    node: InternalNode<K>,
}

struct PathToLeaf<K, V> {
    path: Vec<PathStep<K>>,
    leaf_id: u64,
    leaf: LeafNode<K, V>,
}

struct SplitResult<K> {
    separator: K,
    new_node_id: u64,
    left_count: u64,
    right_count: u64,
}

impl<K> InternalNode<K> {
    fn insert_child(&mut self, idx: usize, split: &SplitResult<K>)
    where
        K: Clone,
    {
        self.keys.insert(idx, split.separator.clone());
        self.children.insert(idx + 1, split.new_node_id);
        self.counts[idx] = split.left_count;
        self.counts.insert(idx + 1, split.right_count);
    }

    fn absorb_child(&mut self, sep_idx: usize) {
        self.keys.remove(sep_idx);
        self.counts[sep_idx] += self.counts[sep_idx + 1];
        self.counts.remove(sep_idx + 1);
        self.children.remove(sep_idx + 1);
    }
}

use crate::runtime;
use borsh::{BorshDeserialize, BorshSerialize};
use std::marker::PhantomData;
