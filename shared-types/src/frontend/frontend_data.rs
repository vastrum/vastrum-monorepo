#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RpcNodeEndpoint {
    pub addr: SocketAddr,
    pub fingerprint: Fingerprint,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ValidatorInfo {
    pub validator_index: u64,
    pub pub_key: [u8; 32],
    pub stake: u64,
}

/// Data injected by the node into the served HTML via `window.__frontendData`.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FrontendData {
    pub rpc_nodes: Vec<RpcNodeEndpoint>,
    pub helios_checkpoint: String,
    pub genesis_validators: HashMap<u64, ValidatorInfo>,
    pub total_validator_stake: u64,
}

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
pub use vastrum_webrtc_direct_protocol::Fingerprint;
