use crate::runtime;
use borsh::{BorshDeserialize, BorshSerialize};
use std::marker::PhantomData;

#[derive(BorshSerialize, BorshDeserialize, Clone)]
pub struct KvVec<T> {
    nonce: u64,
    #[borsh(skip)]
    _phantom: PhantomData<T>,
}

impl<T> Default for KvVec<T> {
    fn default() -> Self {
        Self { nonce: runtime::next_nonce(), _phantom: PhantomData }
    }
}

impl<T> KvVec<T>
where
    T: BorshSerialize + BorshDeserialize,
{
    pub fn new() -> Self {
        return Self::default();
    }

    fn length_key(&self) -> String {
        return format!("n.{}.length", self.nonce);
    }

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

    pub fn is_empty(&self) -> bool {
        let is_empty = self.length() == 0;
        return is_empty;
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

    //not exposed to contract in order to avoid creating sparse vecs with holes
    pub(crate) fn delete(&self, index: u64) {
        runtime::kv_delete(&self.element_key(index));
    }
}
