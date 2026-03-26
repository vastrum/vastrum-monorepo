#[derive(Serialize, Deserialize, Clone)]
pub struct PrivateKey {
    key: x25519_dalek::StaticSecret,
}
impl PrivateKey {
    pub fn diffie_hellman(&self, target_pub_key: x25519::PublicKey) -> [u8; 32] {
        let shared = self.key.diffie_hellman(&target_pub_key.key);
        return shared.to_bytes();
    }
    pub fn public_key(&self) -> PublicKey {
        let key = x25519_dalek::PublicKey::from(&self.key);
        PublicKey { key }
    }
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        let key = x25519_dalek::StaticSecret::from(bytes);
        return PrivateKey { key };
    }
    pub fn to_bytes(&self) -> &[u8; 32] {
        return self.key.as_bytes();
    }

    pub fn from_seed(seed: u64) -> Self {
        let mut bytes = [0u8; 32];
        bytes[24..32].copy_from_slice(&seed.to_be_bytes());

        let key = x25519_dalek::StaticSecret::from(bytes);
        return PrivateKey { key };
    }

    pub fn from_rng() -> PrivateKey {
        let mut bytes = [0u8; 32];
        rand::fill(&mut bytes);
        let key = x25519_dalek::StaticSecret::from(bytes);
        return PrivateKey { key };
    }

    pub fn from_sha256_hash(hash: Sha256Digest) -> Self {
        return Self::from_bytes(hash.to_bytes());
    }

    //TODO is correct?
    //https://www.jcraige.com/an-explainer-on-ed25519-clamping
    pub fn from_ed25519(sk: &ed25519::PrivateKey) -> x25519::PrivateKey {
        let mut h = sha2::Sha512::digest(sk.to_bytes());
        h[0] &= 248;
        h[31] &= 127;
        h[31] |= 64;
        let key = x25519_dalek::StaticSecret::from(<[u8; 32]>::try_from(&h[..32]).unwrap());
        return x25519::PrivateKey { key };
    }
    pub fn try_from_string(value: String) -> Option<PrivateKey> {
        let bytes = hex::decode(value).ok()?;
        let bytes: [u8; 32] = bytes.try_into().ok()?;
        Some(PrivateKey::from_bytes(bytes))
    }
}
impl fmt::Display for PrivateKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", hex::encode(self.key.as_bytes()))
    }
}
impl Debug for PrivateKey {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("PrivateKey").field("key", &"KEY_HIDDEN_FOR_SECURITY".to_string()).finish()
    }
}
impl PartialEq for PrivateKey {
    fn eq(&self, other: &Self) -> bool {
        self.key.to_bytes() == other.key.to_bytes()
    }
}
#[derive(Clone, Copy, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub struct PublicKey {
    key: x25519_dalek::PublicKey,
}
impl PublicKey {
    pub fn from_ed25519_public_key(ed_pub: &ed25519::PublicKey) -> Option<PublicKey> {
        let edwards = CompressedEdwardsY(ed_pub.to_bytes()).decompress()?;
        let x25519_pub_key = PublicKey::from_bytes(edwards.to_montgomery().to_bytes());
        return Some(x25519_pub_key);
    }

    pub fn from_bytes(bytes: [u8; 32]) -> PublicKey {
        let key = x25519_dalek::PublicKey::from(bytes);
        return PublicKey { key };
    }
    pub fn to_bytes(&self) -> [u8; 32] {
        return *self.key.as_bytes();
    }
    pub fn from_string(input: &str) -> Option<PublicKey> {
        if let Ok(bytes) = hex::decode(input) {
            if let Ok(data) = bytes.try_into() {
                return Some(PublicKey::from_bytes(data));
            }
        }
        return None;
    }
}

impl fmt::Display for PublicKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", hex::encode(self.key.as_bytes()))
    }
}

impl BorshSerialize for PrivateKey {
    fn serialize<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        let bytes = self.key.to_bytes();
        BorshSerialize::serialize(&bytes, writer)
    }
}
impl BorshDeserialize for PrivateKey {
    fn deserialize_reader<R: Read>(reader: &mut R) -> io::Result<Self> {
        let bytes: [u8; 32] = BorshDeserialize::deserialize_reader(reader)?;
        Ok(PrivateKey::from_bytes(bytes))
    }
}

impl BorshSerialize for PublicKey {
    fn serialize<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        let bytes = self.key.to_bytes();
        BorshSerialize::serialize(&bytes, writer)
    }
}
impl BorshDeserialize for PublicKey {
    fn deserialize_reader<R: Read>(reader: &mut R) -> io::Result<Self> {
        let bytes: [u8; 32] = BorshDeserialize::deserialize_reader(reader)?;

        let key = PublicKey::from_bytes(bytes);
        return Ok(key);
    }
}

#[allow(unused_imports)]
use crate::borsh::*;
use crate::crypto::ed25519;
use crate::crypto::sha256::Sha256Digest;
use crate::crypto::x25519;
use borsh::BorshDeserialize;
use borsh::BorshSerialize;
use curve25519_dalek::edwards::CompressedEdwardsY;
use serde::Deserialize;
use serde::Serialize;
use sha2::Digest;
use std::fmt::{self, Debug, Formatter};
use std::io::{self, Read, Write};
