const PERSISTED_ROUND_STATE: &[u8] = b"persisted_round_state";

#[derive(Clone, Default, BorshSerialize, BorshDeserialize)]
pub struct PersistedRoundState {
    pub height: u64,
    pub current_round: u64,
    pub skip_certs: Vec<SkipCertificate>,
    pub latest_justify_cert: Option<JustifyCertificate>,
    pub justified_block: Option<Block>,
}

impl Db {
    pub fn write_round_state(&self, state: &PersistedRoundState) {
        let bytes = state.encode();
        self.put(cf::VOTE_STATE, PERSISTED_ROUND_STATE, bytes);
    }

    pub fn read_round_state(&self) -> Option<PersistedRoundState> {
        let bytes = self.get(cf::VOTE_STATE, PERSISTED_ROUND_STATE)?;
        PersistedRoundState::try_from_slice(&bytes).ok()
    }
}

use super::{Db, cf};
use crate::consensus::types::{Block, JustifyCertificate, SkipCertificate};
use borsh::{BorshDeserialize, BorshSerialize};
use vastrum_shared_types::borsh::BorshExt;
