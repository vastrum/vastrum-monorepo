#[derive(BorshSerialize, BorshDeserialize, Debug, Eq, PartialEq, Clone)]
pub struct SiteCall {
    pub site_id: Sha256Digest,
    pub args: Vec<u8>,
}

#[allow(unused_imports)]
use crate::borsh::*;
use crate::crypto::sha256::Sha256Digest;
use borsh::{BorshDeserialize, BorshSerialize};
