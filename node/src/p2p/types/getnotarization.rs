#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub struct GetNotarizationRequest {
    pub slot_height: u64,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub struct GetNotarizationState {
    pub height: u64,
    pub votes: Vec<Notarization>,
    pub hash: Sha256Digest,
    pub notarization_type: GetNotarizationNotarizationType,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub struct GetNotarizationReply {
    pub notarizations: Vec<GetNotarizationState>,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub enum GetNotarizationNotarizationType {
    Block,
    Nullification,
    NoneYet,
}

use crate::consensus::types::Notarization;
use borsh::{BorshDeserialize, BorshSerialize};
#[allow(unused_imports)]
use shared_types::borsh::BorshExt;
use shared_types::crypto::sha256::Sha256Digest;
