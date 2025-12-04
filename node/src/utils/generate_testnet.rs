pub fn generate_local_test_network_single_bootstrapping_node(peers: u64) -> Vec<TestNetNode> {
    let mut test_net_nodes: Vec<TestNetNode> = vec![];

    let mut port_number_current = 8021;

    let bootstrapping_node = TestNetNode {
        keystore: keystore::keyset::insecure_generate_new_static_identity(1),
        endpoint: std::net::SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 8021)),
        node_records: vec![],
    };
    test_net_nodes.push(bootstrapping_node.clone());
    port_number_current += 1;

    for i in 2..=peers {
        let port_number = port_number_current;
        let keystore = keystore::keyset::insecure_generate_new_static_identity(i);
        port_number_current += 1;

        let node_records = vec![NodeRecord {
            p2p_key: bootstrapping_node.keystore.p2p_key.public_key().clone(),
            endpoint: bootstrapping_node.endpoint.clone(),
            state: PeerHistory::BootstrappingNode,
            session_state: PeerSessionState::None,
        }];

        let test_net_node = TestNetNode {
            keystore: keystore,
            endpoint: std::net::SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, port_number)),
            node_records: node_records,
        };
        test_net_nodes.push(test_net_node);
    }
    return test_net_nodes;
}
#[derive(Clone, Debug)]
pub struct TestNetNode {
    pub keystore: Keystore,
    pub endpoint: SocketAddr,
    pub node_records: Vec<NodeRecord>,
}

use crate::{
    keystore::{self, keyset::Keystore},
    p2p::domon::{NodeRecord, PeerHistory, PeerSessionState},
};
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
