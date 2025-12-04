#[derive(BorshSerialize, BorshDeserialize, Clone, PartialEq, Debug)]
pub struct BlockConsensusHeader {
    pub block_hash: Sha256Digest,
    pub height: u64,
}

#[derive(BorshSerialize, BorshDeserialize, Clone, PartialEq)]
pub struct BlockData {
    pub height: u64,
    pub transactions: Vec<Transaction>,
    pub previous_block_hash: Sha256Digest,
}
impl BlockData {
    pub fn calculate_block_hash(&self) -> Sha256Digest {
        let block_bytes = self.encode();
        return sha256::hash(&block_bytes);
    }
}

#[derive(BorshSerialize, BorshDeserialize, Clone, PartialEq, Debug)]
pub struct Block {
    pub height: u64,
    pub transactions: Vec<Transaction>,
    pub previous_block_hash: Sha256Digest,
    pub slot_leader_signature: ed25519::Signature,
}
#[derive(BorshSerialize, BorshDeserialize, Clone, PartialEq, Debug)]
pub struct Notarization {
    pub validator_index: u64,
    pub signature: ed25519::Signature,
}

#[derive(BorshSerialize, BorshDeserialize, Clone, PartialEq, Debug)]
pub struct NotarizedBlock {
    pub height: u64,
    pub transactions: Vec<Transaction>,
    pub previous_block_hash: Sha256Digest,
    pub slot_leader_signature: ed25519::Signature,
    pub votes: Vec<Notarization>,
}
impl NotarizedBlock {
    pub fn calculate_block_hash(&self) -> Sha256Digest {
        let block_data = BlockData {
            height: self.height,
            transactions: self.transactions.clone(),
            previous_block_hash: self.previous_block_hash,
        };

        return block_data.calculate_block_hash();
    }
}

#[derive(BorshSerialize, BorshDeserialize, Clone, PartialEq, Debug)]
pub struct NotarizedNullification {
    pub height: u64,
    pub votes: Vec<Notarization>,
}

impl Block {
    pub fn calculate_block_hash(&self) -> Sha256Digest {
        let block_data = BlockData {
            height: self.height,
            transactions: self.transactions.clone(), //todo do not clone here
            previous_block_hash: self.previous_block_hash,
        };
        return block_data.calculate_block_hash();
    }
}

//just used for calculating hash, not actual comms
#[derive(BorshSerialize, BorshDeserialize, Clone, PartialEq)]
pub struct NullificationDigest {
    pub namespace: [u8; 20],
    pub slot_height: u64,
}
impl NullificationDigest {
    pub fn calculate_hash(&self) -> Sha256Digest {
        let bytes = borsh::to_vec(&self).unwrap();
        return hash(&bytes);
    }
}

#[derive(BorshDeserialize, BorshSerialize, Clone, PartialEq, Debug)]
pub enum SlotState {
    Block(NotarizedBlock), //can be view or finalized
    Nullification(NotarizedNullification),
}
use borsh::{BorshDeserialize, BorshSerialize};
#[allow(unused_imports)]
use shared_types::borsh::*;
use shared_types::{
    crypto::{
        ed25519,
        sha256::{self, Sha256Digest, hash},
    },
    types::execution::transaction::Transaction,
};
