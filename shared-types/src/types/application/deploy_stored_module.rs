#[derive(BorshSerialize, BorshDeserialize, Debug, Eq, PartialEq, Clone)]
pub struct DeployStoredModuleCall {
    pub module_id: Sha256Digest,
    pub constructor_calldata: Vec<u8>,
}

#[allow(unused_imports)]
use crate::borsh::*;
use crate::crypto::sha256::Sha256Digest;
use borsh::{BorshDeserialize, BorshSerialize};
