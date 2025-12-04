#[derive(BorshSerialize, BorshDeserialize)]
pub struct GetBlockRequest {
    pub slot_height: u64,
}

#[derive(BorshSerialize, BorshDeserialize)]
pub struct GetBlockReply {
    pub blocks: Vec<Block>,
}

use crate::consensus::types::Block;
use borsh::{BorshDeserialize, BorshSerialize};
#[allow(unused_imports)]
use shared_types::borsh::*;
