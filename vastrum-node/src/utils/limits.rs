use std::time::Duration;

pub const MAX_PROXY_BODY_SIZE: usize = 10 * 1024 * 1024; // 10MB

pub const MAX_MEMPOOL_SIZE: usize = 100 * 1024 * 1024; // 100MB

pub const MAX_ROUND_LOOKAHEAD: u64 = 100;
pub const MAX_SLOT_LOOKAHEAD: u64 = 100;

pub const ROUND_TIMEOUT: Duration = Duration::from_secs(3);
pub const LONG_ROUND_TIMEOUT: Duration = Duration::from_secs(12);

pub const MAX_FRAME_SIZE: usize = 5 * 1024 * 1024; // 5MB
pub const MAX_INBOUND_VALIDATORS: usize = 10_000;
pub const MAX_OUTBOUND_VALIDATORS: usize = 10_000;
pub const MAX_INBOUND_NORMAL: usize = 20;
pub const MAX_OUTBOUND_NORMAL: usize = 20;
pub const MAX_PEER_RECORDS: usize = 100;
