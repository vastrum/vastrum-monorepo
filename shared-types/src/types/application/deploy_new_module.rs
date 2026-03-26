#[derive(BorshSerialize, BorshDeserialize, Debug, Eq, PartialEq, Clone)]
pub struct DeployNewModuleCall {
    pub wasm_data: Vec<u8>,
    pub constructor_calldata: Vec<u8>,
}

#[allow(unused_imports)]
use crate::borsh::*;
use borsh::{BorshDeserialize, BorshSerialize};
