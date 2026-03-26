use crate::runtime;
use borsh::{BorshDeserialize, BorshSerialize};
use sha2::{Digest, Sha256};
use std::marker::PhantomData;

#[derive(BorshSerialize, BorshDeserialize, Clone)]
pub struct KvMap<K, V> {
    nonce: u64,
    #[borsh(skip)]
    _phantom: PhantomData<(K, V)>,
}

//always allocate next_nonce() to ensure kvmap nonce wont get lost in certain nested cases
//where would expect not to need to get and set to store newly gained nonce state
impl<K, V> Default for KvMap<K, V> {
    fn default() -> Self {
        Self { nonce: runtime::next_nonce(), _phantom: PhantomData }
    }
}

impl<K, V> KvMap<K, V>
where
    K: BorshSerialize,
    V: BorshSerialize + BorshDeserialize,
{
    pub fn new() -> Self {
        return Self::default();
    }

    fn data_key(&self, key: &K) -> String {
        let hash = Sha256::digest(borsh::to_vec(key).unwrap());
        let key = format!("n.{}.{:x}", self.nonce, hash);
        return key;
    }

    pub fn get(&self, key: &K) -> Option<V> {
        let bytes = runtime::kv_get(&self.data_key(key));
        if bytes.is_empty() {
            return None;
        } else {
            return Some(borsh::from_slice(&bytes).unwrap());
        }
    }

    pub fn set(&self, key: &K, value: V) {
        runtime::kv_insert(&self.data_key(key), &borsh::to_vec(&value).unwrap());
    }

    pub fn remove(&self, key: &K) {
        runtime::kv_delete(&self.data_key(key));
    }

    pub fn contains(&self, key: &K) -> bool {
        let contains_value = self.get(key).is_some();
        return contains_value;
    }
}
