#[derive(BorshDeserialize, BorshSerialize, Debug, Clone)]
pub enum Message {
    Statement { payload: Vec<u8> },
    Request { id: u64, payload: Vec<u8> },
    Response { id: u64, payload: Vec<u8> },
    Ping { nonce: u64 },
    Pong { nonce: u64 },
}

pub struct Response {
    pub payload: Vec<u8>,
}

pub struct InboundMessage {
    pub payload: Vec<u8>,
    pub respond: Option<ResponseHandle>,
}

pub struct AppInboundMessage {
    pub payload: AppPayload,
    pub respond: Option<ResponseHandle>,
}

use crate::p2p::connection::ResponseHandle;
use crate::p2p::types::payload::AppPayload;
use borsh::{BorshDeserialize, BorshSerialize};
#[allow(unused_imports)]
use vastrum_shared_types::borsh::*;
