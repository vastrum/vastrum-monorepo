#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct SubmitTransactionPayload {
    #[serde(with = "crate::types::rpc::serde_base64::base64_vec")]
    pub transaction_bytes: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct GetPagePayload {
    pub site_identifier: String,
    pub page_path: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct PageResponse {
    #[serde(with = "crate::types::rpc::serde_base64::base64_vec")]
    pub brotli_html_content: Vec<u8>,
    pub site_id: Sha256Digest,
    pub page_path: String,
    pub state_proof: StateProof,
}

#[derive(Clone, Debug, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub enum GetPageResult {
    Ok(PageResponse),
    Err(ProvedReadError),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct GetLatestBlockHeightResponse {
    pub height: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct GetKeyValuePayload {
    pub site_id: Sha256Digest,
    pub key: String,
    pub height_lock: Option<u64>,
}

#[derive(Clone, Debug, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct StateProof {
    pub proof: jmt::proof::SparseMerkleProof<sha2::Sha256>,
    pub block_header: BlockHeader,
    pub round: u64,
    pub finalization_votes: Vec<(u64, ed25519::Signature)>,
}

#[derive(Clone, Debug, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct GetKeyValueResponse {
    #[serde(with = "crate::types::rpc::serde_base64::base64_vec")]
    pub value: Vec<u8>,
    pub state_proof: StateProof,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub enum ProvedReadError {
    OutsideRetentionWindow,
    ProofUnavailable,
    SiteNotFound,
    PageNotFound,
}

#[derive(Clone, Debug, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub enum GetKeyValueResult {
    Ok(GetKeyValueResponse),
    Err(ProvedReadError),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct GetSiteIDIsDeployed {
    pub site_id: Sha256Digest,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct GetSiteIDIsDeployedResponse {
    pub result: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct GetTxHashIsIncluded {
    pub tx_hash: Sha256Digest,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct GetTxHashIsIncludedResponse {
    pub included: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct ResolveDomainRequest {
    pub domain: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct ResolveDomainResponse {
    pub site_id: Option<Sha256Digest>,
}

#[derive(BorshSerialize, BorshDeserialize)]
pub struct RpcRequest {
    pub id: u64,
    pub route: String,
    pub body: Vec<u8>,
}

#[derive(BorshSerialize, BorshDeserialize)]
pub struct RpcResponse {
    pub id: u64,
    pub body: RpcBody,
}

#[derive(BorshSerialize, BorshDeserialize)]
pub enum RpcBody {
    Success(Vec<u8>),
    Error(String),
}

#[derive(Clone, Debug, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
pub struct EthProxyRequest {
    pub url: String,
    pub method: String,
    pub body: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
pub struct EthProxyResponse {
    pub status: u16,
    pub body: Vec<u8>,
    pub content_type: String,
}

#[allow(unused_imports)]
use crate::borsh::*;
use crate::crypto::{ed25519, sha256::Sha256Digest};
use crate::types::consensus::BlockHeader;
use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};
