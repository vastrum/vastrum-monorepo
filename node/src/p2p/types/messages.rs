#[derive(BorshDeserialize, BorshSerialize, Clone, PartialEq, Debug)]
pub struct PingContent {
    pub from_p2p_key: ed25519::PublicKey, //p2p key of pinger
    pub unix_timestamp: u64,
    pub to_p2p_key: ed25519::PublicKey,
    pub listening_port: u16,
}
impl PingContent {
    pub fn calculate_hash(&self) -> Sha256Digest {
        let bytes = self.encode();
        return sha256::hash(&bytes);
    }
}

#[derive(BorshDeserialize, BorshSerialize, Clone, Debug)]
pub struct PingMessage {
    pub signature: ed25519::Signature,
    pub content: PingContent,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct PongContent {
    pub from_p2p_key: ed25519::PublicKey, //p2p key of ponger
    pub unix_timestamp: u64,
    pub to_p2p_key: ed25519::PublicKey,
    pub listening_port: u16,
    pub ping_message_hash: Sha256Digest,
}
impl PongContent {
    pub fn calculate_hash(&self) -> Sha256Digest {
        let bytes = self.encode();
        return hash(&bytes);
    }
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct PongMessage {
    pub signature: ed25519::Signature,
    pub content: PongContent,
}

// Message types
#[derive(BorshDeserialize, BorshSerialize, Debug, Clone)]
pub enum MessageType {
    Request,
    Response,
    Statement,
    Ping,
    Pong,
}

#[derive(BorshDeserialize, BorshSerialize, Debug, Clone)]
pub struct Message {
    pub id: u64,
    pub message_type: MessageType,
    pub payload: Vec<u8>,
}
pub struct ReceivedMessage {
    pub message: Message,
    pub from_peer_connection: Arc<PeerConnection>,
}

use crate::p2p::peer_connection::PeerConnection;
use borsh::{BorshDeserialize, BorshSerialize};
#[allow(unused_imports)]
use shared_types::borsh::*;
use shared_types::crypto::{
    ed25519,
    sha256::{self, Sha256Digest, hash},
};
use std::sync::Arc;
