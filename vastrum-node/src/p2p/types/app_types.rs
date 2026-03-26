#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub struct GetSlotRequest {
    pub height: u64,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct GetSlotReply {
    pub slot: Option<FinalizedBlock>,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub struct GetRoundRequest {
    pub height: u64,
    pub round: u64,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct GetRoundReply {
    pub cert: Option<Certificate>,
}

#[derive(BorshDeserialize, BorshSerialize, Debug, Clone, PartialEq)]
pub struct GetPeersReply {
    pub peers: Vec<KnownPeer>,
}

use crate::consensus::types::{Certificate, FinalizedBlock};
use crate::p2p::peer_manager::KnownPeer;
use borsh::{BorshDeserialize, BorshSerialize};
#[allow(unused_imports)]
use vastrum_shared_types::borsh::BorshExt;
