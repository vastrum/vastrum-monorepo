#[derive(BorshSerialize, BorshDeserialize, Clone, PartialEq)]
pub struct Transaction {
    pub pub_key: ed25519::PublicKey,
    pub signature: ed25519::Signature,
    pub calldata: Vec<u8>,
    pub pow_nonce: u64,
}
impl fmt::Debug for Transaction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Transaction")
            .field("pub_key", &self.pub_key)
            .field("signature", &self.signature)
            .field("pow_nonce", &self.pow_nonce)
            .finish()
    }
}
impl Transaction {
    pub fn calculate_pow_hash(&self) -> Sha256Digest {
        let pow = ProofOfWorkStruct { pow_nonce: self.pow_nonce, pub_key: self.pub_key };
        let encoded = borsh::to_vec(&pow).unwrap();
        return sha256::hash(&encoded);
    }
    pub fn calculate_calldata_hash(&self) -> Sha256Digest {
        return sha256::hash(&self.calldata);
    }
    pub fn calculate_txhash(&self) -> Sha256Digest {
        let bytes = borsh::to_vec(&self).unwrap();
        return sha256::hash(&bytes);
    }
}

#[derive(BorshSerialize, BorshDeserialize)]
pub struct ProofOfWorkStruct {
    pow_nonce: u64,
    pub_key: PublicKey,
}
#[allow(unused_imports)]
use crate::borsh::*;
use crate::crypto::{
    ed25519::{self, PublicKey},
    sha256::{self, Sha256Digest},
};
use borsh::{BorshDeserialize, BorshSerialize};
use std::fmt;
