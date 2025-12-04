use borsh::{BorshDeserialize, BorshSerialize};
#[allow(unused_imports)]
use shared_types::borsh::*;
// Message types
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub enum PayloadType {
    VoteBlock,
    VoteNull,
    BlockProposal,
    GetNotarizationRequest,
    GetNotarizationReply,
    GetBlockRequest,
    GetBlockReply,
    GetPeersRequest,
    GetPeersReply,
    TransactionGossip,
    TransactionSubmitBroadcast,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct Payload {
    pub payload_type: PayloadType,
    pub content: Vec<u8>,
}
