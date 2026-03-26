/// Single node network used to test smart contract runtime
pub async fn start_localnet() {
    unsafe { std::env::set_var("VASTRUM_LOCALNET", "1") };
    let keystore = generate_localnet();
    let rpc_node = RpcNodeEndpoint {
        addr: (local_network_ip(), WEBRTC_PORT).into(),
        fingerprint: keystore.dtls_key.fingerprint(),
    };
    let db = Arc::new(Db::new());
    let config = NodeConfig {
        keystore,
        peers: vec![],
        run_rpc_node: true,
        genesis_epoch_state: genesis_epoch_state(),
        rpc_nodes: vec![rpc_node],
    };
    ValidatorStateMachine::start_node(db, config).await;
}

pub async fn start_node_production(keystore_path: PathBuf, run_rpc: bool) {
    utils::logging::setup_logging();
    let keystore = Keystore::load_or_create(&keystore_path);
    let db = Arc::new(Db::open(Db::default_path()));
    let config = NodeConfig {
        keystore,
        peers: genesis_bootstrap_peers(),
        run_rpc_node: run_rpc,
        genesis_epoch_state: genesis_epoch_state(),
        rpc_nodes: genesis_rpc_nodes(),
    };
    ValidatorStateMachine::start_node(db, config).await;
}

#[macro_use]
pub mod utils;
pub mod block_indexer;
pub mod consensus;
pub mod db;
pub mod execution;
pub mod keystore;
pub mod p2p;
pub mod rpc;

use crate::{
    keystore::keyset::Keystore,
    utils::genesis::{
        generate_localnet, genesis_bootstrap_peers, genesis_epoch_state, genesis_rpc_nodes,
    },
};
use consensus::validator_state_machine::{NodeConfig, ValidatorStateMachine};
use db::Db;
use std::{path::PathBuf, sync::Arc};
use vastrum_shared_types::frontend::frontend_data::RpcNodeEndpoint;
use vastrum_shared_types::ports::WEBRTC_PORT;
use vastrum_webrtc_direct_server::local_network_ip;
