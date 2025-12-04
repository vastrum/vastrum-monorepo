#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub enum TransactionType {
    Call,
    DeployNewComponent,
    AddComponent,
    DeployStoredComponent,
    RegisterDomain,
}

#[derive(BorshSerialize, BorshDeserialize, Clone)]
pub struct TransactionData {
    pub transaction_type: TransactionType,
    pub calldata: Vec<u8>,
}
impl fmt::Debug for TransactionData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TransactionData").field("transaction_type", &self.transaction_type).finish()
    }
}

impl TransactionData {
    pub fn calculate_hash(&self) -> Sha256Digest {
        let bytes = borsh::to_vec(self).unwrap();
        return sha256::hash(&bytes);
    }
}
#[allow(unused_imports)]
use crate::borsh::*;
use crate::crypto::sha256::{self, Sha256Digest};
use borsh::{BorshDeserialize, BorshSerialize};
use std::fmt;
