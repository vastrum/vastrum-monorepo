use borsh::{BorshDeserialize, BorshSerialize};
use vastrum_rpc_client::{RpcClient, RpcProvider};
use sha2::{Digest, Sha256};
use std::fmt;
use std::io;
use std::marker::PhantomData;
use std::sync::Arc;

pub struct KvMap<K, V> {
    nonce: u64,
    client: Arc<RpcClient>,
    _phantom: PhantomData<(K, V)>,
}

impl<K, V> Clone for KvMap<K, V> {
    fn clone(&self) -> Self {
        Self { nonce: self.nonce, client: self.client.clone(), _phantom: PhantomData }
    }
}

impl<K, V> fmt::Debug for KvMap<K, V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("KvMap").field("nonce", &self.nonce).finish_non_exhaustive()
    }
}

impl<K, V> BorshDeserialize for KvMap<K, V> {
    fn deserialize_reader<R: io::Read>(reader: &mut R) -> io::Result<Self> {
        //to pass rpc client to child kv data types as they need it to get()
        //very complicated but works
        let nonce = u64::deserialize_reader(reader)?;
        let client = crate::get_deser_client();
        return Ok(Self { nonce, client, _phantom: PhantomData });
    }
}

impl<K, V> KvMap<K, V>
where
    K: BorshSerialize,
    V: BorshDeserialize,
{
    pub fn new(nonce: u64, client: Arc<RpcClient>) -> Self {
        return Self { nonce, client, _phantom: PhantomData };
    }

    fn data_key(&self, key: &K) -> String {
        let hash = Sha256::digest(borsh::to_vec(key).unwrap());
        let key = format!("n.{}.{:x}", self.nonce, hash);
        return key;
    }

    pub async fn get(&self, key: &K) -> Option<V> {
        let kv_key = self.data_key(key);
        let bytes = self.client.get_key_value(kv_key).await?;
        if bytes.is_empty() {
            return None;
        } else {
            return crate::with_deser_client(&self.client, || borsh::from_slice(&bytes).ok());
        }
    }

    pub async fn contains(&self, key: &K) -> bool {
        return self.get(key).await.is_some();
    }
}
