#[derive(BorshSerialize, BorshDeserialize, Debug, Eq, PartialEq, Clone)]
pub struct DomainData {
    pub site_id: Sha256Digest,
    pub domain_name: String,
}
#[allow(unused_imports)]
use crate::borsh::*;
use crate::crypto::sha256::Sha256Digest;
use borsh::{BorshDeserialize, BorshSerialize};
