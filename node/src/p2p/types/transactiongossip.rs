#[derive(BorshSerialize, BorshDeserialize)]
pub struct TransactionGossip {
    pub transaction: Transaction,
}

use borsh::{BorshDeserialize, BorshSerialize};
#[allow(unused_imports)]
use shared_types::borsh::*;
use shared_types::types::execution::transaction::Transaction;
