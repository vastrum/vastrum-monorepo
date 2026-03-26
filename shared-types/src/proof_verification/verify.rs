pub fn verify_keyvalue_proof(
    response: &GetKeyValueResponse,
    site_id: Sha256Digest,
    key: &str,
    validators: &HashMap<u64, ValidatorInfo>,
    total_stake: u64,
    current_unix_timestamp: u64,
) -> Result<(), ProofVerificationError> {
    let proof = &response.state_proof;

    let block_hash = proof.block_header.calculate_hash();

    verify_finalization_votes(
        &proof.finalization_votes,
        block_hash,
        proof.block_header.height,
        proof.round,
        validators,
        total_stake,
    )?;

    check_proof_staleness(proof, current_unix_timestamp)?;

    let storage_key = SiteKvStorageKey::new(site_id, key).encode();
    let jmt_key_input =
        JmtKeyInput { cf_namespace: cf_to_namespace_byte("sitekv"), key: &storage_key };
    let key_hash = KeyHash::with::<Sha256>(&borsh::to_vec(&jmt_key_input).unwrap());

    let state_root = proof.block_header.previous_block_state_root;

    let root = RootHash(state_root.to_bytes());

    let need_to_verify_non_existance_proof = response.value.is_empty();
    if need_to_verify_non_existance_proof {
        return Ok(proof.proof.verify_nonexistence(root, key_hash)?);
    } else {
        let value_hash = Sha256::digest(&response.value);
        return Ok(proof.proof.verify_existence(root, key_hash, value_hash.as_slice())?);
    }
}

//TODO currently does not verify domain lookups and site_ids
//currently domain lookup is done server side, to do this would need to do domain lookup locally and then request server
//or server attaches proof of domain lookup
//also site_id is not verified, RPC node can return page for any site_id,
//would need to have separate proven domain lookup
//to verify site_id
pub fn verify_page_proof(
    response: &PageResponse,
    validators: &HashMap<u64, ValidatorInfo>,
    total_stake: u64,
    current_unix_timestamp: u64,
) -> Result<(), ProofVerificationError> {
    let proof = &response.state_proof;

    let state_root = proof.block_header.previous_block_state_root;
    let block_hash = proof.block_header.calculate_hash();

    verify_finalization_votes(
        &proof.finalization_votes,
        block_hash,
        proof.block_header.height,
        proof.round,
        validators,
        total_stake,
    )?;

    check_proof_staleness(proof, current_unix_timestamp)?;

    let storage_key = PageStorageKey::new(response.site_id, &response.page_path).encode();
    let jmt_key_input =
        JmtKeyInput { cf_namespace: cf_to_namespace_byte("page"), key: &storage_key };
    let key_hash = KeyHash::with::<Sha256>(&borsh::to_vec(&jmt_key_input).unwrap());

    let root = RootHash(state_root.to_bytes());

    if response.brotli_html_content.is_empty() {
        return Ok(proof.proof.verify_nonexistence(root, key_hash)?);
    } else {
        let page_value = Page {
            site_id: response.site_id,
            path: response.page_path.clone(),
            brotli_html_content: response.brotli_html_content.clone(),
        };
        let value_hash = Sha256::digest(page_value.encode());
        return Ok(proof.proof.verify_existence(root, key_hash, value_hash.as_slice())?);
    }
}

fn check_proof_staleness(
    proof: &StateProof,
    current_unix_timestamp: u64,
) -> Result<(), ProofVerificationError> {
    let block_ts = proof.block_header.timestamp;
    if current_unix_timestamp > block_ts {
        let age = current_unix_timestamp - block_ts;
        if age > MAX_PROOF_AGE_SECS {
            return Err(ProofVerificationError::StaleProof { block_ts, age });
        }
    } else {
        let ahead = block_ts - current_unix_timestamp;
        if ahead > MAX_PROOF_FUTURE_SECS {
            return Err(ProofVerificationError::FutureTimestamp { block_ts, ahead });
        }
    }
    return Ok(());
}

fn verify_finalization_votes(
    votes: &[(u64, ed25519::Signature)],
    block_hash: Sha256Digest,
    height: u64,
    round: u64,
    validators: &HashMap<u64, ValidatorInfo>,
    total_stake: u64,
) -> Result<(), ProofVerificationError> {
    let mut seen_validators = std::collections::HashSet::new();
    let mut verified_stake = 0;

    for (validator_index, signature) in votes {
        //validator_already_signed
        if !seen_validators.insert(validator_index) {
            continue;
        }
        let Some(validator) = validators.get(validator_index) else {
            continue;
        };
        let vote_data =
            ValidatorVoteData { vote_type: VoteType::Finalize(block_hash), height, round };
        let Some(pub_key) = ed25519::PublicKey::try_from_bytes(validator.pub_key) else {
            continue;
        };
        if pub_key.verify_sig(vote_data.calculate_hash(), *signature) {
            verified_stake += validator.stake;
        }
    }

    //66%
    if verified_stake * 3 <= total_stake * 2 {
        return Err(ProofVerificationError::InsufficientStake {
            verified: verified_stake,
            total: total_stake,
        });
    }
    return Ok(());
}

use super::ProofVerificationError;
use crate::borsh::BorshExt;
use crate::crypto::ed25519;
use crate::crypto::sha256::Sha256Digest;
use crate::frontend::frontend_data::ValidatorInfo;
use crate::limits::{MAX_PROOF_AGE_SECS, MAX_PROOF_FUTURE_SECS};
use crate::types::consensus::{ValidatorVoteData, VoteType};
use crate::types::rpc::types::{GetKeyValueResponse, PageResponse, StateProof};
use crate::types::storage::{
    JmtKeyInput, Page, PageStorageKey, SiteKvStorageKey, cf_to_namespace_byte,
};
use jmt::{KeyHash, RootHash};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
