use borsh::BorshDeserialize;
use vastrum_rpc_client::{RpcClient, RpcProvider};
use std::fmt;
use std::io;
use std::marker::PhantomData;
use std::sync::Arc;

pub struct KvVec<T> {
    nonce: u64,
    client: Arc<RpcClient>,
    _phantom: PhantomData<T>,
}

impl<T> Clone for KvVec<T> {
    fn clone(&self) -> Self {
        Self { nonce: self.nonce, client: self.client.clone(), _phantom: PhantomData }
    }
}

impl<T> fmt::Debug for KvVec<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("KvVec").field("nonce", &self.nonce).finish_non_exhaustive()
    }
}

impl<T> BorshDeserialize for KvVec<T> {
    fn deserialize_reader<R: io::Read>(reader: &mut R) -> io::Result<Self> {
        //to pass rpc client to child kv data types as they need it to get()
        //very complicated but works
        let nonce = u64::deserialize_reader(reader)?;
        let client = crate::get_deser_client();
        return Ok(Self { nonce, client, _phantom: PhantomData });
    }
}

impl<T> KvVec<T>
where
    T: BorshDeserialize,
{
    pub fn new(nonce: u64, client: Arc<RpcClient>) -> Self {
        return Self { nonce, client, _phantom: PhantomData };
    }

    fn length_key(&self) -> String {
        return format!("n.{}.length", self.nonce);
    }

    fn element_key(&self, index: u64) -> String {
        return format!("n.{}.{}", self.nonce, index);
    }

    pub async fn length(&self) -> u64 {
        let bytes = self.client.get_key_value(self.length_key()).await.unwrap_or_default();
        let length = borsh::from_slice(&bytes).unwrap_or(0);
        return length;
    }

    pub async fn is_empty(&self) -> bool {
        let is_empty = self.length().await == 0;
        return is_empty;
    }

    pub async fn get(&self, index: u64) -> Option<T> {
        let key = self.element_key(index);
        let bytes = self.client.get_key_value(key).await?;
        if bytes.is_empty() {
            return None;
        } else {
            //to pass rpc client to child kv data types as they need it to get()
            //very complicated but works
            return crate::with_deser_client(&self.client, || borsh::from_slice(&bytes).ok());
        }
    }

    pub async fn get_at_height(&self, index: u64, height: u64) -> Option<T> {
        let key = self.element_key(index);
        let bytes = self.client.get_key_value_at_height(key, height).await?;
        if bytes.is_empty() {
            return None;
        } else {
            return crate::with_deser_client(&self.client, || borsh::from_slice(&bytes).ok());
        }
    }
}
