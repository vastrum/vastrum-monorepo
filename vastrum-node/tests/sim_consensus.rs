#![cfg(madsim)]

//stressed conditions to test liveness
const NETWORK_LATENCY_200_TO_500_MS: Range<Duration> = ms(200)..ms(500);
const PACKET_LOSS_10_PERCENT: f64 = 0.10;

//more normal network conditions to test throghput in happy path
const NETWORK_LATENCY_80_TO_250_MS: Range<Duration> = ms(80)..ms(250);
const PACKET_LOSS_HALF_PERCENT: f64 = 0.005;

//latency above round timeout (3s), proposals will always arrive too late and nodes will always skip
const NETWORK_LATENCY_ABOVE_ROUND_TIMEOUT: Range<Duration> = ms(4000)..ms(6000);

#[madsim::test]
async fn five_nodes_reach_consensus() {
    let [node_1, node_2, node_3, node_4, node_5] = start_madsim_localnet::<5>();
    set_network_conditions(NETWORK_LATENCY_200_TO_500_MS, PACKET_LOSS_10_PERCENT);

    tokio::time::sleep(Duration::from_secs(360)).await;

    assert!(node_1.latest_finalized_height() >= 100, "network didn't reach consensus");

    assert_chain_consistency(&[node_1, node_2, node_3, node_4, node_5]);
}

#[madsim::test]
async fn three_live_two_offline_network_does_not_reach_consensus() {
    let [node_1, node_2, _node_3, _node_4, node_5] = start_madsim_localnet();
    set_network_conditions(NETWORK_LATENCY_200_TO_500_MS, PACKET_LOSS_10_PERCENT);

    node_1.kill_node();
    node_2.kill_node();

    //remaining 3 should halt
    assert_node_is_halted(&node_5).await;
}

#[madsim::test]
async fn network_can_recover_from_no_online_nodes() {
    let [node_1, node_2, node_3, node_4, node_5] = start_madsim_localnet();
    set_network_conditions(NETWORK_LATENCY_200_TO_500_MS, PACKET_LOSS_10_PERCENT);

    node_1.block_all_network_comms();
    node_2.block_all_network_comms();
    node_3.block_all_network_comms();
    node_4.block_all_network_comms();
    node_5.block_all_network_comms();

    //keep nodes offline for 90s
    tokio::time::sleep(Duration::from_secs(90)).await;

    node_1.unblock_all_network_comms();
    node_2.unblock_all_network_comms();
    node_3.unblock_all_network_comms();
    node_4.unblock_all_network_comms();
    node_5.unblock_all_network_comms();

    tokio::time::sleep(Duration::from_secs(90)).await;
    assert_chain_consistency(&[node_1, node_2, node_3, node_4, node_5]);
}

#[madsim::test]
async fn node_can_sync_from_genesis() {
    let [node_1, node_2, node_3, node_4, node_5] = start_madsim_localnet();
    set_network_conditions(NETWORK_LATENCY_200_TO_500_MS, PACKET_LOSS_10_PERCENT);

    node_4.kill_node();

    //progress chain for 90 seconds
    tokio::time::sleep(Duration::from_secs(90)).await;

    //node_4 should sync
    node_4.restart_node();
    tokio::time::sleep(Duration::from_secs(90)).await;

    assert_chain_consistency(&[node_1, node_2, node_3, node_4, node_5]);
}

#[madsim::test]
async fn network_partition_and_recovery() {
    let [node_1, node_2, node_3, node_4, node_5] = start_madsim_localnet();
    set_network_conditions(NETWORK_LATENCY_200_TO_500_MS, PACKET_LOSS_10_PERCENT);

    //let network reach consensus
    tokio::time::sleep(Duration::from_secs(180)).await;

    //partition node 3
    node_3.block_all_network_comms();

    //remaining 4 nodes should still have 400/500 stake > 333 threshold and should progress
    assert_node_is_finalizing_blocks(&node_1).await;

    //reconnect node 3
    node_3.unblock_all_network_comms();
    assert_node_is_finalizing_blocks(&node_3).await;

    //repeated partitions
    for _ in 0..50 {
        node_3.block_all_network_comms();
        tokio::time::sleep(Duration::from_secs(5)).await;
        node_3.unblock_all_network_comms();
        tokio::time::sleep(Duration::from_secs(5)).await;
    }

    //recovery window for node 3 to sync missed blocks
    tokio::time::sleep(Duration::from_secs(600)).await;

    assert_chain_consistency(&[node_1, node_2, node_3, node_4, node_5]);
}

#[madsim::test]
async fn minority_partition_halts_then_recovers() {
    let [node_1, node_2, node_3, node_4, node_5] = start_madsim_localnet();
    set_network_conditions(NETWORK_LATENCY_200_TO_500_MS, PACKET_LOSS_10_PERCENT);

    //let network reach consensus
    tokio::time::sleep(Duration::from_secs(180)).await;

    //partition 2 nodes: remaining 3 have 300/500 stake < 333 threshold
    node_4.block_all_network_comms();
    node_5.block_all_network_comms();

    //consensus should halt
    assert_node_is_halted(&node_1).await;

    //reconnect: consensus should resume (needs time to resync round state)
    node_4.unblock_all_network_comms();
    node_5.unblock_all_network_comms();

    assert_node_is_finalizing_blocks(&node_1).await;

    assert_chain_consistency(&[node_1, node_2, node_3, node_4, node_5]);
}

#[madsim::test]
async fn node_crash_and_restart() {
    let [node_1, node_2, node_3, node_4, node_5] = start_madsim_localnet();
    set_network_conditions(NETWORK_LATENCY_200_TO_500_MS, PACKET_LOSS_10_PERCENT);

    //let network reach consensus
    tokio::time::sleep(Duration::from_secs(180)).await;

    node_3.kill_node();

    //remaining 4 should continue
    assert_node_is_finalizing_blocks(&node_1).await;

    node_3.restart_node();

    assert_node_is_finalizing_blocks(&node_3).await;

    assert_chain_consistency(&[node_1, node_2, node_3, node_4, node_5]);
}

#[madsim::test]
async fn block_production_rate_happy_path_network() {
    let [node_1, node_2, node_3, node_4, node_5] = start_madsim_localnet();

    set_network_conditions(NETWORK_LATENCY_80_TO_250_MS, PACKET_LOSS_HALF_PERCENT);
    let min_block_rate_per_sec_5_nodes = 1.0;
    let min_block_rate_per_sec_4_nodes = 0.6;
    let testing_time = 300;

    //assert block production rate when 5/5 nodes online is 1 per second at least
    assert_block_production_rate(&node_1, testing_time, min_block_rate_per_sec_5_nodes).await;

    //no block should have taken more then 5 consensus rounds to finalize
    assert_no_slow_rounds(&node_1, 5);

    //assert block production rate when 4/5 nodes online is 0.6 per second at least
    node_3.block_all_network_comms();

    assert_block_production_rate(&node_1, testing_time, min_block_rate_per_sec_4_nodes).await;
    // node 3 is still partitioned, only check the 4 active nodes
    assert_chain_consistency(&[node_1, node_2, node_4, node_5]);
}

#[madsim::test]
async fn block_production_rate_degraded_network() {
    let [node_1, node_2, node_3, node_4, node_5] = start_madsim_localnet();
    set_network_conditions(NETWORK_LATENCY_200_TO_500_MS, PACKET_LOSS_10_PERCENT);
    let min_block_production_rate = 0.5;
    let testing_time = 300;
    //assert block production rate is at least 0.5 per second in degraded network conditions
    assert_block_production_rate(&node_1, testing_time, min_block_production_rate).await;

    assert_chain_consistency(&[node_1, node_2, node_3, node_4, node_5]);
}

#[madsim::test]
async fn block_production_rate_rolling_freezes() {
    let nodes = start_madsim_localnet::<5>();
    set_network_conditions(NETWORK_LATENCY_80_TO_250_MS, PACKET_LOSS_HALF_PERCENT);

    for iteration in 0..10 {
        let first = madsim::rand::thread_rng().gen_range(0..5);
        let mut frozen = vec![first];

        let freeze_two_nodes = iteration % 2 == 0;
        if freeze_two_nodes {
            let mut second = madsim::rand::thread_rng().gen_range(0..5);
            while second == first {
                second = madsim::rand::thread_rng().gen_range(0..5);
            }
            frozen.push(second);
        }

        for &idx in &frozen {
            nodes[idx].block_all_network_comms();
        }

        tokio::time::sleep(Duration::from_secs(10)).await;

        for &idx in &frozen {
            nodes[idx].unblock_all_network_comms();
        }

        tokio::time::sleep(Duration::from_secs(15)).await;
    }

    assert_block_production_rate(&nodes[0], 180, 0.05).await;

    assert_chain_consistency(&nodes);
}
#[madsim::test]
async fn rolling_restart_all_nodes() {
    let nodes = start_madsim_localnet::<5>();
    set_network_conditions(NETWORK_LATENCY_80_TO_250_MS, PACKET_LOSS_HALF_PERCENT);

    //wait until chain has progressed some
    loop {
        tokio::time::sleep(Duration::from_secs(5)).await;
        if nodes[0].latest_finalized_height() >= 20 {
            break;
        }
    }

    //rolling restarts
    for i in 0..5 {
        nodes[i].kill_node();
        tokio::time::sleep(Duration::from_secs(30)).await;
        nodes[i].restart_node();
        tokio::time::sleep(Duration::from_secs(30)).await;
    }

    //wait for all nodes to sync
    tokio::time::sleep(Duration::from_secs(120)).await;

    assert_chain_consistency(&nodes);
}

//tests that after a complete crash
//network can still recover by each node rebroadcasting their
//write ahead disk written vote
#[madsim::test]
async fn write_ahead_log_vote_recovery_after_all_nodes_crash_works() {
    let [node_1, node_2, node_3, node_4, node_5] = start_madsim_localnet();
    set_network_conditions(NETWORK_LATENCY_200_TO_500_MS, PACKET_LOSS_10_PERCENT);

    //let network finalize some blocks
    tokio::time::sleep(Duration::from_secs(180)).await;

    //partition all nodes so no votes propagate
    node_1.block_all_network_comms();
    node_2.block_all_network_comms();
    node_3.block_all_network_comms();
    node_4.block_all_network_comms();
    node_5.block_all_network_comms();

    //wait past skip vote timeout (3s) so all nodes vote skip locally
    //but cant receive each others votes so no cert is formed
    tokio::time::sleep(Duration::from_secs(5)).await;

    //kill all nodes  each has voted skip but no skip cert exists
    node_1.kill_node();
    node_2.kill_node();
    node_3.kill_node();
    node_4.kill_node();
    node_5.kill_node();

    tokio::time::sleep(Duration::from_secs(5)).await;

    //unclog and restart
    node_1.unblock_all_network_comms();
    node_2.unblock_all_network_comms();
    node_3.unblock_all_network_comms();
    node_4.unblock_all_network_comms();
    node_5.unblock_all_network_comms();

    node_1.restart_node();
    node_2.restart_node();
    node_3.restart_node();
    node_4.restart_node();
    node_5.restart_node();

    //consensus should resume without vote persistence this deadlocks
    //because all nodes have Recovered state for the round they voted in
    //but no skip cert exists to advance past it
    assert_node_is_finalizing_blocks(&node_1).await;

    assert_chain_consistency(&[node_1, node_2, node_3, node_4, node_5]);
}

//tests that after a full network crash in the middle of a consensus round
//that the network can still recover
//using disk written consensus round certs
#[madsim::test]
async fn mid_round_cert_recovery_after_all_nodes_crash_mid_round_works() {
    let [node_1, node_2, node_3, node_4, node_5] = start_madsim_localnet();

    //latency > round timeout (3s) so proposals always arrive too late
    //nodes skip every round, no block is ever justified or finalized
    set_network_conditions(NETWORK_LATENCY_ABOVE_ROUND_TIMEOUT, 0.0);

    //wait until all nodes have gone through at least 5 skip rounds
    loop {
        let nodes = [&node_1, &node_2, &node_3, &node_4, &node_5];
        let min_round = nodes.iter().map(|n| n.current_round()).min().unwrap();
        if min_round >= 5 {
            break;
        }
        tokio::time::sleep(Duration::from_secs(2)).await;
    }

    assert_eq!(node_1.latest_finalized_height(), 0, "should not have finalized any block");
    assert_eq!(node_2.latest_finalized_height(), 0, "should not have finalized any block");
    assert_eq!(node_3.latest_finalized_height(), 0, "should not have finalized any block");
    assert_eq!(node_4.latest_finalized_height(), 0, "should not have finalized any block");
    assert_eq!(node_5.latest_finalized_height(), 0, "should not have finalized any block");

    node_1.kill_node();
    node_2.kill_node();
    node_3.kill_node();
    node_4.kill_node();
    node_5.kill_node();

    tokio::time::sleep(Duration::from_secs(5)).await;

    node_1.unblock_all_network_comms();
    node_2.unblock_all_network_comms();
    node_3.unblock_all_network_comms();
    node_4.unblock_all_network_comms();
    node_5.unblock_all_network_comms();
    set_network_conditions(NETWORK_LATENCY_200_TO_500_MS, PACKET_LOSS_10_PERCENT);

    node_1.restart_node();
    node_2.restart_node();
    node_3.restart_node();
    node_4.restart_node();
    node_5.restart_node();

    //consensus should resume, without vote persistence this deadlocks
    assert_node_is_finalizing_blocks(&node_1).await;

    assert_chain_consistency(&[node_1, node_2, node_3, node_4, node_5]);
}

#[derive(Clone)]
struct SimNode {
    db: Arc<Db>,
    id: NodeId,
}

impl SimNode {
    fn block_all_network_comms(&self) {
        madsim::net::NetSim::current().clog_node(self.id);
    }

    fn unblock_all_network_comms(&self) {
        madsim::net::NetSim::current().unclog_node(self.id);
    }

    fn kill_node(&self) {
        Handle::current().kill(self.id);
    }

    fn restart_node(&self) {
        Handle::current().restart(self.id);
    }

    fn latest_finalized_height(&self) -> u64 {
        self.db.read_latest_finalized_height()
    }

    fn block_hash_at(
        &self,
        height: u64,
    ) -> Option<vastrum_shared_types::crypto::sha256::Sha256Digest> {
        self.db.read_block(height).map(|fc| fc.block.calculate_hash())
    }

    fn current_round(&self) -> u64 {
        self.db.read_round_state().map(|s| s.current_round).unwrap_or(0)
    }
}
fn start_madsim_localnet<const AMOUNT_OF_NODES: usize>() -> [SimNode; AMOUNT_OF_NODES] {
    assert!(AMOUNT_OF_NODES == 5);
    let handle = Handle::current();
    let (testnet, epoch_state) = generate_sim_test_network(AMOUNT_OF_NODES as u64);

    let mut nodes = vec![];

    for (i, test_node) in testnet.iter().enumerate() {
        let ip = test_node.endpoint.ip();
        let test_node = test_node.clone();
        let epoch_state = epoch_state.clone();

        let db = Arc::new(Db::new());
        let db_for_test = db.clone();

        let node = handle
            .create_node()
            .name(format!("validator-{}", i + 1))
            .ip(ip)
            .init(move || {
                let test_node = test_node.clone();
                let epoch_state = epoch_state.clone();
                let db = db.clone();
                async move {
                    let config = NodeConfig {
                        keystore: test_node.keystore.clone(),
                        peers: test_node.node_records.clone(),
                        run_rpc_node: false,
                        genesis_epoch_state: epoch_state,
                        rpc_nodes: vec![],
                    };
                    ValidatorStateMachine::start_node(db, config).await;
                }
            })
            .build();

        nodes.push(SimNode { db: db_for_test, id: node.id() });
    }

    let Ok(nodes) = nodes.try_into() else { unreachable!() };
    nodes
}

fn assert_chain_consistency(nodes: &[SimNode]) {
    let mut max_height: u64 = 0;
    for node in nodes {
        let h = node.latest_finalized_height();
        if h > max_height {
            max_height = h;
        }
    }
    let check_up_to = max_height - 5;
    assert!(
        check_up_to >= 2,
        "need at least 2 finalized blocks to check consistency, got {}",
        check_up_to
    );

    let first_node = &nodes[0];
    for height in 1..=check_up_to {
        let expected = first_node.block_hash_at(height).expect("node should have block");
        for (i, node) in nodes.iter().enumerate().skip(1) {
            let actual = node.block_hash_at(height).expect("node should have block");
            assert_eq!(
                expected,
                actual,
                "fork detected at height {}: node 1 has {:?}, node {} has {:?}",
                height,
                expected,
                i + 1,
                actual
            );
        }
    }
}

async fn assert_block_production_rate(node: &SimNode, duration_secs: u64, min_blocks_per_sec: f64) {
    let height_before = node.latest_finalized_height();
    tokio::time::sleep(Duration::from_secs(duration_secs)).await;
    let height_after = node.latest_finalized_height();
    let blocks_produced = height_after.saturating_sub(height_before);
    let actual_rate = blocks_produced as f64 / duration_secs as f64;
    assert!(actual_rate >= min_blocks_per_sec, "block production rate rate requirement not met");
}

fn assert_no_slow_rounds(node: &SimNode, max_rounds_per_block: u64) {
    for height in 1..=node.latest_finalized_height() {
        let block = node.db.read_block(height).unwrap();
        assert!(block.round < max_rounds_per_block, "block took too many rounds to finalize");
    }
}

async fn assert_node_is_finalizing_blocks(node: &SimNode) -> u64 {
    let duration_secs = 360;
    let min_blocks = 10;
    let height_before = node.latest_finalized_height();
    tokio::time::sleep(Duration::from_secs(duration_secs)).await;
    let height_after = node.latest_finalized_height();
    assert!(height_after > height_before + min_blocks, "blockchain did not progress when expected");

    return height_after;
}

async fn assert_node_is_halted(node: &SimNode) {
    tokio::time::sleep(Duration::from_secs(3)).await;
    let height_before = node.latest_finalized_height();
    tokio::time::sleep(Duration::from_secs(90)).await;
    let height_after = node.latest_finalized_height();
    assert!(height_after == height_before, "blockchain should have halted",);
}

fn set_network_conditions(latency: Range<Duration>, packet_loss: f64) {
    madsim::net::NetSim::current().update_config(|config| {
        config.send_latency = latency;
        config.packet_loss_rate = packet_loss;
    });
}
const fn ms(v: u64) -> Duration {
    Duration::from_millis(v)
}

#[derive(Clone, Debug)]
struct TestNetNode {
    keystore: Keystore,
    endpoint: SocketAddr,
    node_records: Vec<KnownPeer>,
}

fn generate_sim_test_network(node_count: u64) -> (Vec<TestNetNode>, EpochState) {
    let bootstrap_key = vastrum_node::keystore::keyset::insecure_generate_new_static_identity(1);
    let bootstrap_endpoint =
        SocketAddr::from((Ipv4Addr::new(10, 0, 0, 1), vastrum_shared_types::ports::P2P_PORT));
    let bootstrap_peer =
        KnownPeer { p2p_key: bootstrap_key.p2p_key.public_key(), endpoint: bootstrap_endpoint };

    let mut nodes = Vec::new();
    let mut epoch_state = EpochState::new();

    for i in 1..=node_count {
        let node = TestNetNode {
            keystore: vastrum_node::keystore::keyset::insecure_generate_new_static_identity(i),
            endpoint: SocketAddr::from((
                Ipv4Addr::new(10, 0, 0, i as u8),
                vastrum_shared_types::ports::P2P_PORT,
            )),
            node_records: if i == 1 { vec![] } else { vec![bootstrap_peer] },
        };
        epoch_state.add_registered_validator(
            node.keystore.validator_private_key.public_key(),
            node.keystore.p2p_key.public_key(),
            100,
        );
        nodes.push(node);
    }

    (nodes, epoch_state)
}

use madsim::rand::Rng;
use madsim::runtime::Handle;
use madsim::task::NodeId;
use std::net::{Ipv4Addr, SocketAddr};
use std::ops::Range;
use std::sync::Arc;
use std::time::Duration;
use vastrum_node::consensus::validator_state_machine::{
    EpochState, NodeConfig, ValidatorStateMachine,
};
use vastrum_node::db::Db;
use vastrum_node::keystore::keyset::Keystore;
use vastrum_node::p2p::peer_manager::KnownPeer;
