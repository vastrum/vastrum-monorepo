#[derive(BorshSerialize, BorshDeserialize, Debug, Eq, PartialEq, Clone)]
pub struct Page {
    pub site_id: Sha256Digest,
    pub path: String,
    pub content: String,
}

use borsh::{BorshDeserialize, BorshSerialize};
#[allow(unused_imports)]
use shared_types::borsh::*;
use shared_types::crypto::sha256::Sha256Digest;
