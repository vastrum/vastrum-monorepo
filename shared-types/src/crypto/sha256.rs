//Based upon https://github.com/commonwarexyz/monorepo/tree/main/cryptography/src/sha256

pub fn sha256_hash(data: &[u8]) -> Sha256Digest {
    let result = Sha256::digest(data);
    return Sha256Digest { data: result.into() };
}

#[derive(
    BorshSerialize,
    BorshDeserialize,
    Clone,
    Copy,
    Default,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    Hash,
    Serialize,
    Deserialize,
    Tsify,
)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct Sha256Digest {
    data: [u8; 32],
}
impl From<[u8; 32]> for Sha256Digest {
    fn from(data: [u8; 32]) -> Self {
        Sha256Digest { data }
    }
}
impl Sha256Digest {
    pub fn from_u64(data: u64) -> Sha256Digest {
        let mut bytes = [0u8; 32];
        bytes[24..32].copy_from_slice(&data.to_be_bytes());
        return Sha256Digest { data: bytes };
    }
    pub fn to_bytes(&self) -> [u8; 32] {
        return self.data;
    }
    pub fn from_rng() -> Sha256Digest {
        let mut bytes = [0u8; 32];
        rand::fill(&mut bytes);
        return Sha256Digest::from(bytes);
    }
    pub fn from_string(input: &str) -> Option<Sha256Digest> {
        if let Some(bytes) = base32::decode(Alphabet::Rfc4648Lower { padding: false }, input) {
            let data: [u8; 32] = bytes.try_into().ok()?;
            return Some(Sha256Digest::from(data));
        }
        return None;
    }
}
impl fmt::Display for Sha256Digest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", base32::encode(Alphabet::Rfc4648Lower { padding: false }, &self.data))
    }
}
impl fmt::Debug for Sha256Digest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self}")
    }
}

#[allow(unused_imports)]
use crate::borsh::*;
use base32::Alphabet;
use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fmt;
use tsify::Tsify;
