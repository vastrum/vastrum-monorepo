//Todo implement namespacing
//Based upon https://github.com/commonwarexyz/monorepo/tree/main/cryptography/src/ed25519
#[derive(Clone, Debug)]
pub struct PrivateKey {
    key: ed25519_consensus::SigningKey,
}
impl PrivateKey {
    pub fn sign(&self, data: &[u8]) -> Signature {
        let signature = self.key.sign(data);
        return Signature { signature };
    }
    pub fn sign_hash(&self, hash: Sha256Digest) -> Signature {
        let signature = self.key.sign(&hash.to_vec());
        return Signature { signature };
    }

    pub fn public_key(&self) -> PublicKey {
        PublicKey { key: self.key.verification_key() }
    }
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        let key = ed25519_consensus::SigningKey::from(bytes);
        return PrivateKey { key };
    }

    pub fn from_seed(seed: u64) -> Self {
        let mut bytes = [0u8; 32];
        bytes[24..32].copy_from_slice(&seed.to_be_bytes());

        let key = ed25519_consensus::SigningKey::from(bytes);
        return PrivateKey { key };
    }
}
#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct PublicKey {
    key: VerificationKey,
}
impl PublicKey {
    pub fn verify_signature(&self, data: &[u8], signature: &Signature) -> bool {
        self.key.verify(&signature.signature, &data).is_ok()
    }
    pub fn verify_signature_hash(&self, data: Sha256Digest, signature: &Signature) -> bool {
        self.key.verify(&signature.signature, &data.encode()).is_ok()
    }
    pub fn try_from_bytes(bytes: [u8; 32]) -> Option<PublicKey> {
        let key = ed25519_consensus::VerificationKey::try_from(bytes);
        if let Ok(key) = key {
            return Some(PublicKey { key });
        } else {
            return None;
        }
    }
}

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub struct Signature {
    signature: ed25519_consensus::Signature,
}
impl Signature {
    pub fn from_bytes(bytes: [u8; 64]) -> Signature {
        let signature = ed25519_consensus::Signature::from(bytes);
        return Signature { signature: signature };
    }
}

impl BorshSerialize for PrivateKey {
    fn serialize<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        let bytes = self.key.to_bytes();
        BorshSerialize::serialize(&bytes, writer)
    }
}
impl BorshDeserialize for PrivateKey {
    fn deserialize_reader<R: std::io::Read>(reader: &mut R) -> std::io::Result<Self> {
        let bytes: [u8; 32] = BorshDeserialize::deserialize_reader(reader)?;
        Ok(PrivateKey::from_bytes(bytes))
    }
}

impl BorshSerialize for PublicKey {
    fn serialize<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        let bytes = self.key.to_bytes();
        BorshSerialize::serialize(&bytes, writer)
    }
}
impl BorshDeserialize for PublicKey {
    fn deserialize_reader<R: std::io::Read>(reader: &mut R) -> std::io::Result<Self> {
        let bytes: [u8; 32] = BorshDeserialize::deserialize_reader(reader)?;

        let result = PublicKey::try_from_bytes(bytes);
        if let Some(key) = result {
            return Ok(key);
        } else {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Invalid verification key: {:#?}", bytes),
            ));
        }
    }
}

impl BorshSerialize for Signature {
    fn serialize<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        let bytes = self.signature.to_bytes();
        BorshSerialize::serialize(&bytes, writer)
    }
}
impl BorshDeserialize for Signature {
    fn deserialize_reader<R: std::io::Read>(reader: &mut R) -> std::io::Result<Self> {
        let bytes: [u8; 64] = BorshDeserialize::deserialize_reader(reader)?;
        Ok(Signature::from_bytes(bytes))
    }
}

#[allow(unused_imports)]
use crate::borsh::*;
use crate::crypto::sha256::Sha256Digest;
use borsh::BorshDeserialize;
use borsh::BorshSerialize;
use ed25519_consensus::VerificationKey;
