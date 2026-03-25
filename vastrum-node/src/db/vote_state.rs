use super::{Db, cf};
use crate::consensus::types::Block;
use borsh::BorshDeserialize;
use vastrum_shared_types::{borsh::BorshExt, crypto::sha256::Sha256Digest};

const LAST_JUSTIFY_VOTE: &[u8] = b"last_justify_vote";
const LAST_COMMIT_VOTE: &[u8] = b"last_commit_vote";

#[derive(borsh::BorshSerialize, borsh::BorshDeserialize, Clone)]
pub struct PersistedFinalizeVote {
    pub block_hash: Sha256Digest,
}

#[derive(borsh::BorshSerialize, borsh::BorshDeserialize, Clone)]
pub struct PersistedJustifyVote {
    pub block: Block,
}

#[derive(borsh::BorshSerialize, borsh::BorshDeserialize, Clone)]
pub enum LatestCommitVoteState {
    Finalize(PersistedFinalizeVote),
    Skip,
    NoneYet,
}

#[derive(borsh::BorshSerialize, borsh::BorshDeserialize, Clone)]
pub enum LatestJustifyVoteState {
    Justify(PersistedJustifyVote),
    NoneYet,
}

#[derive(borsh::BorshSerialize, borsh::BorshDeserialize, Clone)]
pub struct LatestJustifyVote {
    pub height: u64,
    pub round: u64,
    pub state: LatestJustifyVoteState,
}

#[derive(borsh::BorshSerialize, borsh::BorshDeserialize, Clone)]
pub struct LatestCommitVote {
    pub height: u64,
    pub round: u64,
    pub state: LatestCommitVoteState,
}

impl Db {
    pub fn write_ahead_log_finalize_vote(&self, height: u64, round: u64, block_hash: Sha256Digest) {
        let persisted = PersistedFinalizeVote { block_hash };
        let state = LatestCommitVoteState::Finalize(persisted);

        let vote = LatestCommitVote { height, round, state };
        self.put(cf::VOTE_STATE, LAST_COMMIT_VOTE, vote.encode());
    }

    pub fn write_ahead_log_skip_vote(&self, height: u64, round: u64) {
        let state = LatestCommitVoteState::Skip;
        let vote = LatestCommitVote { height, round, state };

        self.put(cf::VOTE_STATE, LAST_COMMIT_VOTE, vote.encode());
    }

    pub fn write_ahead_log_justify_vote(&self, height: u64, round: u64, block: Block) {
        let persisted = PersistedJustifyVote { block };
        let state = LatestJustifyVoteState::Justify(persisted);
        let vote = LatestJustifyVote { height, round, state };
        self.put(cf::VOTE_STATE, LAST_JUSTIFY_VOTE, vote.encode());
    }

    pub fn read_last_justify_vote(&self) -> LatestJustifyVote {
        let Some(vote) = self.get(cf::VOTE_STATE, LAST_JUSTIFY_VOTE) else {
            return LatestJustifyVote {
                height: 0,
                round: 0,
                state: LatestJustifyVoteState::NoneYet,
            };
        };
        let state = LatestJustifyVote::try_from_slice(&vote).unwrap();
        return state;
    }

    pub fn read_last_commit_vote(&self) -> LatestCommitVote {
        let Some(vote) = self.get(cf::VOTE_STATE, LAST_COMMIT_VOTE) else {
            return LatestCommitVote { height: 0, round: 0, state: LatestCommitVoteState::NoneYet };
        };
        let state = LatestCommitVote::try_from_slice(&vote).unwrap();
        return state;
    }
}
