use crate::limits::{MAX_PROOF_AGE_SECS, MAX_PROOF_FUTURE_SECS};

#[derive(thiserror::Error, Debug)]
pub enum ProofVerificationError {
    #[error("insufficient finalization stake: {verified}/{total}")]
    InsufficientStake { verified: u64, total: u64 },
    #[error("merkle proof failed")]
    MerkleProof(#[from] anyhow::Error),
    #[error("stale proof: block timestamp {block_ts} is {age}s old (max {MAX_PROOF_AGE_SECS}s)")]
    StaleProof { block_ts: u64, age: u64 },
    #[error(
        "future timestamp: block timestamp {block_ts} is {ahead}s ahead (max {MAX_PROOF_FUTURE_SECS}s)"
    )]
    FutureTimestamp { block_ts: u64, ahead: u64 },
}
