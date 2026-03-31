use crate::crypto::{ed25519, encryption::CipherText, sha256::Sha256Digest, x25519};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum RpcMethod {
    GetKeyValue,
    MakeCall,
    MakeAuthenticatedCall,
    GetPrivateSalt,
    GetSitePubKey,
    GetTxHashIsIncluded,
    GetCurrentPath,
    UpdateCurrentPath,
    EthRpcRequest,
    StarknetRpcRequest,
    GetKeyValueBySiteId,
    OpenExternalUrl,
    GetLatestBlockHeight,
    GetSitePrivateKey,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum RpcMethodHostToIFrame {
    Response,
    PageNavigationEvent,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RpcRequest {
    pub request_id: u64,
    pub method: RpcMethod,
    pub params: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RpcResponse {
    pub request_id: u64,
    pub method: RpcMethodHostToIFrame,
    pub params: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetPublicKey {}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetPublicKeyResult {
    pub public_key: ed25519::PublicKey,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Getx25519PublicKey {}

#[derive(Serialize, Deserialize, Debug)]
pub struct Getx25519PublicKeyResult {
    pub public_key: x25519::PublicKey,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EncryptMessageRequest {
    pub content: String,
    pub target_pub_key: x25519::PublicKey,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EncryptMessageResponse {
    pub cipher_text: CipherText,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DecryptMessageRequest {
    pub cipher_text: CipherText,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DecryptMessageResponse {
    pub content: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetKeyValueRequest {
    pub key: String,
    #[serde(default)]
    pub height: Option<u64>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetLatestBlockHeight {}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetLatestBlockHeightResponse {
    pub height: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetKeyValueResponse {
    #[serde(with = "crate::types::rpc::serde_base64::base64_vec")]
    pub value: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetKeyValueBySiteIdRequest {
    pub site_id: Sha256Digest,
    pub key: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MakeCallRequest {
    #[serde(with = "crate::types::rpc::serde_base64::base64_vec")]
    pub call_data: Vec<u8>,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct MakeCallResponse {
    pub tx_hash: Sha256Digest,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MakeAuthCallRequest {
    #[serde(with = "crate::types::rpc::serde_base64::base64_vec")]
    pub call_data: Vec<u8>,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct MakeAuthCallResponse {
    pub tx_hash: Sha256Digest,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetPrivateSalt {}
#[derive(Serialize, Deserialize, Debug)]
pub struct GetPrivateSaltResponse {
    pub salt: Sha256Digest,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetPubKey {}
#[derive(Serialize, Deserialize, Debug)]
pub struct GetPubKeyResponse {
    pub pub_key: ed25519::PublicKey,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetPrivateKeyRpc {}
#[derive(Serialize, Deserialize, Debug)]
pub struct GetPrivateKeyResponse {
    pub private_key: ed25519::PrivateKey,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetTXHashIsConfirmed {
    pub tx_hash: Sha256Digest,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct GetTXHashIsConfirmedResponse {
    pub is_finalized: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetCurrentPath {}
#[derive(Serialize, Deserialize, Debug)]
pub struct GetCurrentPathResponse {
    pub path: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UpdateCurrentPath {
    pub path: String,
    pub replace: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UpdateCurrentPathResponse {}

#[derive(Serialize, Deserialize, Debug)]
pub struct PageNavigationEventMessage {
    pub path: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EthRPCRequest {
    pub method: String,
    #[serde(default)]
    pub params: Vec<Value>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct EthRPCResponse {
    pub value_json: Value,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OpenExternalUrlRequest {
    pub url: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OpenExternalUrlResponse {}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetEthRPCRequest {
    pub request: EthRPCRequest,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct GetEthRPCResponse {
    pub eth_rpc_response: EthRPCResponse,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StarknetRPCRequest {
    pub rpc_url: String,
    pub method: String,
    #[serde(default)]
    pub params: Value,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct StarknetRPCResponse {
    pub value_json: Value,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetStarknetRPCRequest {
    pub request: StarknetRPCRequest,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct GetStarknetRPCResponse {
    pub starknet_rpc_response: StarknetRPCResponse,
}
