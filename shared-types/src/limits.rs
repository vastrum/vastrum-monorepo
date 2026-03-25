pub const MAX_TRANSACTIONS_PER_BLOCK: usize = 1000;
pub const MAX_BLOCK_SIZE: usize = 4 * 1024 * 1024; //4mb
pub const VALIDITY_WINDOW: u64 = 300;

pub const MAX_TRANSACTION_SIZE: usize = 4 * 1024 * 1024; //4mb
pub const MAX_DECOMPRESSED_CALLDATA_SIZE: usize = 16 * 1024 * 1024; //16mb

pub const MAX_WASM_MODULE_SIZE: usize = 1 * 1024 * 1024; //1mb
pub const MAX_WASM_MEMORY: usize = 256 * 1024 * 1024; //256mb
pub const MAX_WASM_HOST_BUFFER_SIZE: u32 = MAX_WASM_MEMORY as u32;

pub const KV_RETENTION_WINDOW: u64 = 64;

pub const MAX_RPC_BODY_SIZE: usize = 4 * 1024 * 1024; //4mb

pub const MAX_PROOF_AGE_SECS: u64 = 120;
pub const MAX_PROOF_FUTURE_SECS: u64 = 30;
