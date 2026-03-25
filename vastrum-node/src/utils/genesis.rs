pub fn genesis_epoch_state() -> EpochState {
    let config = genesis_config();
    let mut epoch_state = EpochState::new();
    for v in &config.validators {
        let pub_key = pubkey(&v.validator_pub_key);
        let p2p_key = pubkey(&v.p2p_pub_key);
        epoch_state.add_registered_validator(pub_key, p2p_key, v.stake);
    }
    return epoch_state;
}

pub fn genesis_bootstrap_peers() -> Vec<KnownPeer> {
    let config = genesis_config();
    let mut peers = Vec::new();
    for bp in &config.bootstrap_peers {
        peers.push(KnownPeer {
            p2p_key: pubkey(&bp.p2p_pub_key),
            endpoint: resolve_host(&bp.host, P2P_PORT),
        });
    }
    return peers;
}

pub fn genesis_rpc_nodes() -> Vec<RpcNodeEndpoint> {
    let config = genesis_config();
    let mut nodes = Vec::new();
    for rn in &config.rpc_nodes {
        nodes.push(RpcNodeEndpoint {
            addr: resolve_host(&rn.host, WEBRTC_PORT),
            fingerprint: Fingerprint::from_hex(&rn.fingerprint),
        });
    }
    return nodes;
}

pub fn generate_localnet() -> Keystore {
    return keystore::keyset::insecure_generate_new_static_identity(1);
}

fn pubkey(hex: &str) -> ed25519::PublicKey {
    let bytes = hex::decode(hex).expect("invalid hex in genesis pubkey");
    let bytes: [u8; 32] = bytes.try_into().expect("genesis pubkey must be 32 bytes");
    ed25519::PublicKey::try_from_bytes(bytes).expect("invalid genesis pubkey")
}

fn resolve_host(host: &str, port: u16) -> SocketAddr {
    let addr = format!("{host}:{port}");
    return addr.to_socket_addrs().unwrap().next().unwrap();
}

use crate::consensus::validator_state_machine::EpochState;
use crate::keystore::{self, keyset::Keystore};
use crate::p2p::peer_manager::KnownPeer;
use std::net::{SocketAddr, ToSocketAddrs};
use vastrum_shared_types::crypto::ed25519;
use vastrum_shared_types::frontend::frontend_data::{Fingerprint, RpcNodeEndpoint};
use vastrum_shared_types::genesis::genesis_config;
use vastrum_shared_types::ports::{P2P_PORT, WEBRTC_PORT};
