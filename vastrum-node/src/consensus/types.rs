#[derive(BorshSerialize, BorshDeserialize, Clone, Copy, Debug, PartialEq)]
pub enum ProposalType {
    Reproposal(u64),
    Proposal,
}
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
pub struct Proposal {
    pub round: u64,
    pub proposal_type: ProposalType,
    pub block: Block,
    pub leader_signature: ed25519::Signature,
}
impl Proposal {
    pub fn create_signed(
        round: u64,
        proposal_type: ProposalType,
        block: Block,
        private_key: &ed25519::PrivateKey,
    ) -> Self {
        let proposal_data = ProposalData { round, proposal_type, block: block.clone() };
        let leader_signature = private_key.sign_hash(proposal_data.calculate_hash());

        let proposal = Proposal { leader_signature, round, proposal_type, block };

        return proposal;
    }
    pub fn calculate_hash(&self) -> Sha256Digest {
        let proposal_data = ProposalData {
            round: self.round,
            proposal_type: self.proposal_type,
            block: self.block.clone(),
        };
        proposal_data.calculate_hash()
    }
}
#[derive(BorshSerialize, BorshDeserialize)]
struct ProposalData {
    pub round: u64,
    pub proposal_type: ProposalType,
    pub block: Block,
}
impl ProposalData {
    fn calculate_hash(&self) -> Sha256Digest {
        sha256_hash(&self.encode())
    }
}

#[derive(BorshSerialize, BorshDeserialize, Clone, PartialEq, Debug)]
pub struct Block {
    pub height: u64,
    pub transactions: Vec<Transaction>,
    pub previous_block_hash: Sha256Digest,
    pub timestamp: u64,
    pub previous_block_state_root: Sha256Digest,
}

impl Block {
    pub fn calculate_hash(&self) -> Sha256Digest {
        let transactions_hash = sha256_hash(&borsh::to_vec(&self.transactions).unwrap());
        let header = BlockHeader {
            height: self.height,
            previous_block_hash: self.previous_block_hash,
            timestamp: self.timestamp,
            previous_block_state_root: self.previous_block_state_root,
            transactions_hash,
        };
        header.calculate_hash()
    }
}

pub use vastrum_shared_types::types::consensus::VoteType;

#[derive(BorshSerialize, BorshDeserialize, Clone, PartialEq, Debug)]
pub struct ValidatorVote {
    pub vote: VoteType,
    pub height: u64,
    pub round: u64,

    pub signature: ed25519::Signature,
    pub validator_index: u64,
}
impl ValidatorVote {
    pub fn create_signed(
        vote: VoteType,
        height: u64,
        round: u64,
        validator_index: u64,
        private_key: &ed25519::PrivateKey,
    ) -> Self {
        let vote_data = ValidatorVoteData { vote_type: vote.clone(), height, round };
        let signature = private_key.sign_hash(vote_data.calculate_hash());

        ValidatorVote { vote, height, round, signature, validator_index }
    }

    pub fn hash(&self) -> Sha256Digest {
        ValidatorVoteData { vote_type: self.vote.clone(), height: self.height, round: self.round }
            .calculate_hash()
    }
}

#[derive(BorshSerialize, BorshDeserialize, Clone, PartialEq, Debug)]
pub enum Certificate {
    Skip(SkipCertificate),
    Justify(JustifyCertificate),
    Finalization(FinalizationCertificate),
}

#[derive(BorshSerialize, BorshDeserialize, Clone, PartialEq, Debug)]
pub struct JustifyCertificate {
    pub block_hash: Sha256Digest,
    pub votes: BTreeMap<ValidatorIndex, ed25519::Signature>,
    pub round: u64,
    pub height: u64,
}

#[derive(BorshSerialize, BorshDeserialize, Clone, PartialEq, Debug)]
pub struct SkipCertificate {
    pub votes: BTreeMap<ValidatorIndex, ed25519::Signature>,
    pub round: u64,
    pub height: u64,
}

#[derive(BorshSerialize, BorshDeserialize, Clone, PartialEq, Debug)]
pub struct FinalizationCertificate {
    pub block_hash: Sha256Digest,
    pub votes: BTreeMap<ValidatorIndex, ed25519::Signature>,
    pub round: u64,
    pub height: u64,
}

#[derive(BorshSerialize, BorshDeserialize, Clone, PartialEq, Debug)]
pub struct FinalizedBlock {
    pub block: Block,
    pub votes: BTreeMap<ValidatorIndex, ed25519::Signature>,
    pub round: u64,
}

#[derive(Default, Debug)]
pub struct RoundSyncStateExternal {
    pub latest_justify: Option<JustifyCertificate>,
    pub skip_certs: HashMap<u64, SkipCertificate>,
}

use crate::consensus::validator_state_machine::ValidatorIndex;
use borsh::{BorshDeserialize, BorshSerialize};
#[allow(unused_imports)]
use vastrum_shared_types::borsh::*;
use vastrum_shared_types::types::consensus::{BlockHeader, ValidatorVoteData};
use vastrum_shared_types::{
    crypto::{
        ed25519,
        sha256::{Sha256Digest, sha256_hash},
    },
    types::execution::transaction::Transaction,
};
use std::collections::{BTreeMap, HashMap};
