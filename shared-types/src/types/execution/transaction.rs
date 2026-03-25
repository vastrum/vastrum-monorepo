#[derive(BorshSerialize, BorshDeserialize, Clone, PartialEq)]
pub struct Transaction {
    pub calldata: Vec<u8>,
    pub pub_key: ed25519::PublicKey,
    pub signature: ed25519::Signature,
    pub nonce: u64,
    pub recent_block_height: u64,
}
impl fmt::Debug for Transaction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Transaction")
            .field("pub_key", &self.pub_key)
            .field("signature", &self.signature)
            .field("nonce", &self.nonce)
            .field("recent_block_height", &self.recent_block_height)
            .finish()
    }
}
impl Transaction {
    pub fn calculate_pow_hash(&self) -> Sha256Digest {
        let pow = ProofOfWorkStruct {
            nonce: self.nonce,
            pub_key: self.pub_key,
            recent_block_height: self.recent_block_height,
        };
        let encoded = pow.encode();
        return sha256::sha256_hash(&encoded);
    }
    pub fn calculate_calldata_hash(&self) -> Sha256Digest {
        return sha256::sha256_hash(&self.calldata);
    }
    pub fn calculate_txhash(&self) -> Sha256Digest {
        let bytes = self.encode();
        return sha256::sha256_hash(&bytes);
    }
    pub fn verify_signature(&self) -> bool {
        let calldata_hash = self.calculate_calldata_hash();
        self.pub_key.verify_sig(calldata_hash, self.signature)
    }
    pub fn verify_gas(&self) -> bool {
        self.calldata.len() <= MAX_TRANSACTION_SIZE
    }
}

#[derive(BorshSerialize, BorshDeserialize)]
pub struct ProofOfWorkStruct {
    nonce: u64,
    pub_key: PublicKey,
    recent_block_height: u64,
}
#[allow(unused_imports)]
use crate::borsh::*;
use crate::crypto::{
    ed25519::{self, PublicKey},
    sha256::{self, Sha256Digest},
};
use crate::limits::MAX_TRANSACTION_SIZE;
use borsh::{BorshDeserialize, BorshSerialize};
use std::fmt;
