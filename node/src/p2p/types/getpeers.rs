use borsh::{BorshDeserialize, BorshSerialize};
#[allow(unused_imports)]
use shared_types::borsh::*;

#[derive(BorshDeserialize, BorshSerialize, Debug, Clone, PartialEq)]
pub struct GetPeersEntry {
    pub p2p_key: ed25519::PublicKey,
    pub endpoint: SocketAddr,
}

#[derive(BorshDeserialize, BorshSerialize, Debug, Clone, PartialEq)]
pub struct GetPeersReply {
    pub peers: Vec<GetPeersEntry>,
}

use shared_types::crypto::ed25519;
use std::net::SocketAddr;
