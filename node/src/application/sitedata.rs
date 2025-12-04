#[derive(BorshSerialize, BorshDeserialize, Debug, Eq, PartialEq, Clone)]
pub struct SiteData {
    pub site_id: Sha256Digest,
    pub component_id: Sha256Digest,
}

use borsh::{BorshDeserialize, BorshSerialize};
#[allow(unused_imports)]
use shared_types::borsh::*;
use shared_types::crypto::sha256::Sha256Digest;
