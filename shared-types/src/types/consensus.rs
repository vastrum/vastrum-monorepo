#[derive(BorshSerialize, BorshDeserialize, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct BlockHeader {
    pub height: u64,
    pub previous_block_hash: Sha256Digest,
    pub timestamp: u64,
    pub previous_block_state_root: Sha256Digest,
    pub transactions_hash: Sha256Digest,
}

impl BlockHeader {
    pub fn calculate_hash(&self) -> Sha256Digest {
        sha256_hash(&self.encode())
    }
}

#[derive(BorshSerialize, BorshDeserialize, Clone, PartialEq, Debug)]
pub enum VoteType {
    Justify(Sha256Digest),
    Finalize(Sha256Digest),
    Skip,
}

#[derive(BorshSerialize, BorshDeserialize)]
pub struct ValidatorVoteData {
    pub vote_type: VoteType,
    pub height: u64,
    pub round: u64,
}

impl ValidatorVoteData {
    pub fn calculate_hash(&self) -> Sha256Digest {
        sha256_hash(&self.encode())
    }
}

#[allow(unused_imports)]
use crate::borsh::*;
use crate::crypto::sha256::{Sha256Digest, sha256_hash};
use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};
