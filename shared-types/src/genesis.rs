use crate::frontend::frontend_data::ValidatorInfo;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GenesisConfig {
    pub validators: Vec<GenesisValidator>,
    pub bootstrap_peers: Vec<GenesisBootstrapPeer>,
    pub rpc_nodes: Vec<GenesisRpcNode>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GenesisValidator {
    pub validator_index: u64,
    pub validator_pub_key: String,
    pub p2p_pub_key: String,
    pub stake: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GenesisBootstrapPeer {
    pub p2p_pub_key: String,
    pub host: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GenesisRpcNode {
    pub host: String,
    pub fingerprint: String,
}

pub fn genesis_config() -> GenesisConfig {
    let json = if std::env::var("VASTRUM_LOCALNET").is_ok() {
        include_str!("../genesis-dev.json")
    } else {
        include_str!("../genesis.json")
    };
    serde_json::from_str(json).expect("invalid genesis json")
}

impl GenesisConfig {
    pub fn to_genesis_epoch_state(&self) -> GenesisEpochState {
        let mut validators = HashMap::new();
        let mut total_stake = 0;
        for v in &self.validators {
            let pub_key = hex_to_bytes32(&v.validator_pub_key);
            validators.insert(
                v.validator_index,
                ValidatorInfo { validator_index: v.validator_index, pub_key, stake: v.stake },
            );
            total_stake += v.stake;
        }
        return GenesisEpochState { validators, total_stake };
    }
}

fn hex_to_bytes32(hex: &str) -> [u8; 32] {
    let bytes = hex::decode(hex).expect("invalid hex in genesis config");
    bytes.try_into().expect("expected 32 bytes in genesis config")
}

pub struct GenesisEpochState {
    pub validators: HashMap<u64, ValidatorInfo>,
    pub total_stake: u64,
}

pub fn genesis_epoch_state() -> GenesisEpochState {
    return genesis_config().to_genesis_epoch_state();
}
