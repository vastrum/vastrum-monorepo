#[derive(BorshSerialize, BorshDeserialize, Clone, PartialEq, Debug)]
pub struct BlockVote {
    pub block_hash: Sha256Digest,
    pub slot_height: u64,
    pub signature: ed25519::Signature,
    pub validator_index: u64,
}

#[derive(BorshSerialize, BorshDeserialize, Clone, PartialEq, Debug)]
pub struct NullVote {
    pub slot_height: u64,
    pub signature: ed25519::Signature,
    pub validator_index: u64,
}

pub enum ConsensusMessage {
    BlockVote(super::comms::BlockVote),
    NullVote(super::comms::NullVote),
}

use borsh::{BorshDeserialize, BorshSerialize};
use shared_types::crypto::{ed25519, sha256::Sha256Digest};
