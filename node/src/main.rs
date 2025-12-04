use shared_types::crypto::ed25519;
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    start_testnet().await;
    return Ok(());
}
async fn start_testnet() {
    let node_id = std::env::var("NODE_ID").unwrap().parse::<u16>().unwrap();
    let testnet_network = generate_local_test_network_single_bootstrapping_node(5);

    let _ = tokio::spawn(async move {
        let node_index = node_id - 1;
        let node = testnet_network.get(node_index as usize).unwrap();

        let mut run_rpc_node = false;
        if node_id == 1 {
            run_rpc_node = true;
        }
        start_node(
            node.endpoint,
            node.node_records.clone(),
            node.keystore.private_key.clone(),
            node.keystore.p2p_key.clone(),
            run_rpc_node,
            node_id,
        )
        .await;
    })
    .await;
}
async fn start_node(
    addr: SocketAddr,
    peers: Vec<NodeRecord>,
    private_key: ed25519::PrivateKey,
    p2p_key: ed25519::PrivateKey,
    run_rpc_node: bool,
    node_id: u16,
) {
    clear_old_data(node_id);
    setup_tracing_stdout(node_id);
    let consensus_thread = tokio::spawn(async move {
        let mut node = ValidatorStateMachine::new(private_key, p2p_key, addr);
        return node.run(addr, peers, run_rpc_node).await;
    });

    let _consensus_result = tokio::try_join!(consensus_thread).unwrap();
}

fn clear_old_data(node_id: u16) {
    sitedb::SiteDatabase::remove_db_if_exists(node_id);
    blockchain::BlockchainDatabase::remove_db_if_exists(node_id);
    pagedb::PageDatabase::remove_db_if_exists(node_id);
    application::sql::db::remove_db_if_exists_port();
    domaindb::DomainDatabase::remove_db_if_exists(node_id);
    componentdb::ComponentDatabase::remove_db_if_exists(node_id);
}

fn setup_tracing_stdout(node_id: u16) {
    std::fs::create_dir_all("logs").unwrap();
    let log_file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true) // This clears the file
        .open(format!("logs/node{}.log", node_id))
        .unwrap();

    tracing_subscriber::registry()
        .with(EnvFilter::from_default_env().add_directive(tracing::Level::INFO.into()))
        .with(
            fmt::layer()
                .with_writer(log_file)
                .with_ansi(true)
                .with_target(true)
                .with_thread_ids(true),
        )
        .init();
}
use crate::{
    consensus::minimmit::ValidatorStateMachine,
    db::{blockchain, componentdb, domaindb, pagedb, sitedb},
    p2p::domon::NodeRecord,
    utils::generate_testnet::generate_local_test_network_single_bootstrapping_node,
};
use std::{error::Error, fs::OpenOptions, net::SocketAddr};

pub mod application;
pub mod consensus;
pub mod db;
pub mod execution;
pub mod keystore;
pub mod node;
pub mod p2p;
pub mod rpc;
pub mod utils;
