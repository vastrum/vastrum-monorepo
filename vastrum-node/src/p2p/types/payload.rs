use borsh::{BorshDeserialize, BorshSerialize};
#[allow(unused_imports)]
use vastrum_shared_types::borsh::*;

use crate::consensus::types::{Certificate, Proposal, ValidatorVote};
use crate::p2p::types::app_types::{GetRoundRequest, GetSlotRequest};
use vastrum_shared_types::types::execution::transaction::Transaction;

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub enum NetworkPayload {
    P2pGetPeersRequest,
    App(AppPayload),
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub enum AppPayload {
    Vote(ValidatorVote),
    Proposal(Proposal),
    GetSlotReq(GetSlotRequest),
    GetRoundReq(GetRoundRequest),
    TransactionGossip(Transaction),
    Certificate(Certificate),
}
