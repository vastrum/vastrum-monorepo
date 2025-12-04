//Based upon https://github.com/commonwarexyz/monorepo/tree/main/cryptography/src/sha256

pub fn hash(data: &Vec<u8>) -> Sha256Digest {
    let result = Sha256::digest(data);
    return Sha256Digest { data: result.into() };
}
#[derive(BorshSerialize, BorshDeserialize, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Sha256Digest {
    data: [u8; 32],
}
impl Sha256Digest {
    pub fn from_u64(data: u64) -> Sha256Digest {
        let mut bytes = [0u8; 32];
        bytes[24..32].copy_from_slice(&data.to_be_bytes());
        return Sha256Digest { data: bytes };
    }
    pub fn from(data: [u8; 32]) -> Sha256Digest {
        return Sha256Digest { data: data };
    }
    pub fn to_vec(&self) -> [u8; 32] {
        return self.data;
    }
    pub fn to_string(&self) -> String {
        return hex::encode(self.data);
    }
    pub fn from_string(input: &str) -> Option<Sha256Digest> {
        if let Ok(bytes) = hex::decode(input) {
            if let Ok(data) = bytes.try_into() {
                return Some(Sha256Digest::from(data));
            }
        }
        return None;
    }
}
impl fmt::Debug for Sha256Digest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

#[allow(unused_imports)]
use crate::borsh::*;
use borsh::{BorshDeserialize, BorshSerialize};
use sha2::{Digest, Sha256};
use std::fmt;
