//Todo implement namespacing
//Based upon https://github.com/commonwarexyz/monorepo/tree/main/cryptography/src/ed25519
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PrivateKey {
    key: ed25519_consensus::SigningKey,
}
impl PrivateKey {
    pub fn sign(&self, data: &[u8]) -> Signature {
        let signature = self.key.sign(data);
        return Signature { signature };
    }
    pub fn sign_hash(&self, hash: Sha256Digest) -> Signature {
        let signature = self.key.sign(&hash.to_bytes());
        return Signature { signature };
    }

    pub fn public_key(&self) -> PublicKey {
        PublicKey { bytes: *self.key.verification_key().as_bytes() }
    }
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        let key = ed25519_consensus::SigningKey::from(bytes);
        return PrivateKey { key };
    }
    pub fn to_bytes(&self) -> &[u8; 32] {
        return self.key.as_bytes();
    }

    pub fn from_seed(seed: u64) -> Self {
        let mut bytes = [0u8; 32];
        bytes[24..32].copy_from_slice(&seed.to_be_bytes());

        let key = ed25519_consensus::SigningKey::from(bytes);
        return PrivateKey { key };
    }

    pub fn from_rng() -> PrivateKey {
        let mut bytes = [0u8; 32];
        rand::fill(&mut bytes);
        let key = ed25519_consensus::SigningKey::from(bytes);
        return PrivateKey { key };
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
#[derive(
    Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default, Serialize, Deserialize,
)]
pub struct PublicKey {
    bytes: [u8; 32],
}
impl PublicKey {
    pub fn verify_signature(&self, data: &[u8], signature: Signature) -> bool {
        let Ok(vk) = ed25519_consensus::VerificationKey::try_from(self.bytes) else {
            return false;
        };
        vk.verify(&signature.signature, data).is_ok()
    }
    pub fn verify_sig(&self, data: Sha256Digest, signature: Signature) -> bool {
        self.verify_signature(&data.encode(), signature)
    }
    pub fn try_from_bytes(bytes: [u8; 32]) -> Option<PublicKey> {
        Some(PublicKey { bytes })
    }
    pub fn to_bytes(&self) -> [u8; 32] {
        self.bytes
    }
    pub fn verifying_key(&self) -> ed25519_consensus::VerificationKeyBytes {
        ed25519_consensus::VerificationKeyBytes::from(self.bytes)
    }
}

impl fmt::Display for PublicKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", hex::encode(self.bytes))
    }
}

impl From<PublicKey> for vastrum_runtime_shared::Ed25519PublicKey {
    fn from(pk: PublicKey) -> Self {
        Self { bytes: pk.bytes }
    }
}

impl From<vastrum_runtime_shared::Ed25519PublicKey> for PublicKey {
    fn from(pk: vastrum_runtime_shared::Ed25519PublicKey) -> Self {
        PublicKey { bytes: pk.bytes }
    }
}

impl From<Signature> for vastrum_runtime_shared::Ed25519Signature {
    fn from(sig: Signature) -> Self {
        Self { bytes: sig.inner().to_bytes() }
    }
}

impl From<vastrum_runtime_shared::Ed25519Signature> for Signature {
    fn from(sig: vastrum_runtime_shared::Ed25519Signature) -> Self {
        Signature::from_bytes(sig.bytes)
    }
}

#[derive(Clone, Copy, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub struct Signature {
    signature: ed25519_consensus::Signature,
}

impl Signature {
    pub fn from_bytes(bytes: [u8; 64]) -> Signature {
        let signature = ed25519_consensus::Signature::from(bytes);
        return Signature { signature };
    }

    pub fn inner(&self) -> ed25519_consensus::Signature {
        self.signature
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
        BorshSerialize::serialize(&self.bytes, writer)
    }
}
impl BorshDeserialize for PublicKey {
    fn deserialize_reader<R: Read>(reader: &mut R) -> io::Result<Self> {
        let bytes: [u8; 32] = BorshDeserialize::deserialize_reader(reader)?;
        Ok(PublicKey { bytes })
    }
}

impl BorshSerialize for Signature {
    fn serialize<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        let bytes = self.signature.to_bytes();
        BorshSerialize::serialize(&bytes, writer)
    }
}
impl BorshDeserialize for Signature {
    fn deserialize_reader<R: Read>(reader: &mut R) -> io::Result<Self> {
        let bytes: [u8; 64] = BorshDeserialize::deserialize_reader(reader)?;
        Ok(Signature::from_bytes(bytes))
    }
}

#[allow(unused_imports)]
use crate::borsh::*;
use crate::crypto::sha256::Sha256Digest;
use borsh::BorshDeserialize;
use borsh::BorshSerialize;
use serde::Deserialize;
use serde::Serialize;
use std::fmt;
use std::io::{self, Read, Write};
