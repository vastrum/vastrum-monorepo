# KV Structure

All state in Vastrum is stored in keyvalue storage. 
  
    Key = string
    Value = bytes


This data is then exposed as a dumb keyvalue API to the client.

There are no view functions or other processing, just raw kv reads.

This makes thing such as state hash merkle tree proof inclusions very simple.

It is also probably the only data structure that private information retrieval practically could work with.


However this makes some datastructures and properties harder to achieve.

For example 
-   How to query posts by latest bump time + pagination?
-   How to remove elements from an array without creating unbounded sparse spaces or require expensive recompactation?


Vastrum implements 4 primitives data structures built ontop of raw keyvalue storage.


```rust
kvvec: KvVec<String>
kvmap: KvMap<String, u64>
kvbtree: KvBTree<u64, String>
kvvecbtree: KvVecBTree<u64, ForumPost>
```

KvVec > simple array
```rust
pub fn length(&self) -> u64 
pub fn get(&self, index: u64) -> Option<T> 
pub fn push(&self, value: T) -> u64 
pub fn set(&self, index: u64, value: T) 
```



KvMap > hashmap
```rust
pub fn get(&self, key: &K) -> Option<V> 
pub fn set(&self, key: &K, value: V) 
pub fn remove(&self, key: &K) 
```

KvBtree > a btree implementation using kv db as node storage, allows for ordered lists. This solves the forum bumptime problem.
 ```rust
pub fn length(&self) -> u64 
pub fn is_empty(&self) -> bool 
pub fn get(&self, key: &K) -> Option<V> 
pub fn first(&self) -> Option<(K, V)> 
pub fn last(&self) -> Option<(K, V)> 
pub fn range(&self, start: &K, end: &K) -> Vec<(K, V)> 
pub fn get_ascending_entries(&self, count: usize, offset: usize) -> Vec<(K, V)> 
pub fn get_descending_entries(&self, count: usize, offset: usize) -> Vec<(K, V)> 
pub fn insert(&self, key: K, value: V) 
pub fn remove(&self, key: &K) -> bool
```

![BTree structure](btree.svg)

KvVecBtree > a KvVec + KvBtree used to give you constant ID to access a element with, if you just use a BTree the index changes with every update to the BTree key, the vec allows you to have a constant reference to an underlying element even if you change the sorting key.
![KvVecBTree array structure](kvvecbtree_array.svg)

KvVecBtree gives this API
```rust
pub fn get_ascending_entries(&self, count: usize, offset: usize) -> Vec<V> 
pub fn get_descending_entries(&self, count: usize, offset: usize) -> Vec<V> 
pub fn range(&self, start: &S, end: &S) -> Vec<V> 
pub fn update(&self, id: u64, new_sort_key: S, value: V) 
pub fn remove(&self, id: u64) 
pub fn get(&self, id: u64) -> Option<V> 
pub fn push(&self, sort_key: S, value: V) -> u64 
```

get_ascending_entries and get_descending_entries in particular allows for pagination to be implemented.


## Concrete code example for KvVec

Data is stored like this

```rust
fn length_key(&self) -> String {
    return format!("n.{}.length", self.nonce);
}

fn element_key(&self, index: u64) -> String {
    return format!("n.{}.{}", self.nonce, index);
}
```

The nonce is a global variable to discriminate between different instances of data structures, it can be seen as a pointer in the C language.

Implementation of public API.

```rust
fn element_key(&self, index: u64) -> String {
    return format!("n.{}.{}", self.nonce, index);
}
pub fn length(&self) -> u64 {
    let bytes = runtime::kv_get(&self.length_key());
    if bytes.is_empty() {
        return 0;
    } else {
        let length = borsh::from_slice(&bytes).unwrap();
        return length;
    }
}

pub fn get(&self, index: u64) -> Option<T> {
    let bytes = runtime::kv_get(&self.element_key(index));
    if bytes.is_empty() {
        return None;
    } else {
        let element = borsh::from_slice(&bytes).unwrap();
        return Some(element);
    }
}

pub fn push(&self, value: T) -> u64 {
    let length = self.length();
    let index = length;
    //insert element
    runtime::kv_insert(&self.element_key(index), &borsh::to_vec(&value).unwrap());
    //update length
    let new_length = length + 1;
    runtime::kv_insert(&self.length_key(), &borsh::to_vec(&new_length).unwrap());
    return index;
}

pub fn set(&self, index: u64, value: T) {
    assert!(index < self.length(), "index out of bounds");
    runtime::kv_insert(&self.element_key(index), &borsh::to_vec(&value).unwrap());
}
```

All kv datastructures are serialized like this, just a single u64 nonce used to separate different instances of the datastructure from each other

```rust
#[derive(BorshSerialize, BorshDeserialize, Clone)]
pub struct KvVec<T> {
    nonce: u64,
    #[borsh(skip)]
    _phantom: PhantomData<T>,
}
```
