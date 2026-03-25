#[derive(BorshSerialize, BorshDeserialize, Debug, Eq, PartialEq, Clone)]
pub struct SiteData {
    pub site_id: Sha256Digest,
    pub module_id: Sha256Digest,
}

use borsh::{BorshDeserialize, BorshSerialize};
#[allow(unused_imports)]
use vastrum_shared_types::borsh::*;
use vastrum_shared_types::crypto::sha256::Sha256Digest;
