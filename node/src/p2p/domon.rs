impl P2PDaemon {
    pub async fn run(&self, addr: SocketAddr, initial_peers: Vec<NodeRecord>) {
        //Listen for new connections to this node
        self.listener_handler_run(addr).await;
        //start peer discovery thread
        self.run_peer_handler(initial_peers).await;
    }

    pub async fn send_request_to_random_peer(
        &self,
        payload: Vec<u8>,
    ) -> tokio::io::Result<Message> {
        let peer = self.get_random_peer().await;
        if let Some(peer) = peer {
            return peer.connection.send_request(payload).await;
        } else {
            //todo proper error here
            return Err(Error::new(
                std::io::ErrorKind::Other,
                "could not find peer to send message to",
            ));
        }
    }
    pub fn broadcast_statement_to_all_peers(&self, payload: Vec<u8>) {
        let payload = Message { id: 0, message_type: MessageType::Statement, payload };

        let peers = self.peers.clone();
        tokio::spawn(async move {
            let peer_connections = P2PDaemon::get_all_peers(peers).await;
            for connection in peer_connections {
                let peerstate = connection.peer_connection_state.lock().await;
                if *peerstate == PeerConnectionState::Connected {
                    let _result = connection.send_message(&payload).await;
                }
            }
        });
    }

    async fn listener_handler_run(&self, addr: SocketAddr) {
        let peers = self.peers.clone();
        let message_request_tx = self.p2p_data.message_tx.clone();
        let local_p2p_key = self.p2p_data.p2p_key.clone();
        tokio::spawn(async move {
            P2PDaemon::listen_for_new_connections(addr, peers, message_request_tx, local_p2p_key)
                .await
                .unwrap();
        });
    }
    async fn listen_for_new_connections(
        addr: SocketAddr,
        peers: Arc<Mutex<HashMap<ed25519::PublicKey, Arc<PeerData>>>>,
        message_request_tx: UnboundedSender<ReceivedMessage>,
        local_p2p_key: ed25519::PrivateKey,
    ) -> tokio::io::Result<()> {
        let listener = TcpListener::bind(addr).await?;

        loop {
            let (socket, peer_addr) = listener.accept().await?;
            let message_request_tx = message_request_tx.clone();
            let local_p2p_key = local_p2p_key.clone();
            let peers = peers.clone();
            tokio::spawn(async move {
                let peer_connection = PeerConnection::start_connection_from_listener(
                    socket,
                    message_request_tx,
                    local_p2p_key,
                    addr.port(),
                )
                .await;

                if let Ok(ConnectionResult::Successful(handshake_result)) = peer_connection {
                    let target_p2p_key = handshake_result.p2p_key;

                    //different ports for initiating connections vs receiving
                    let mut peer_addr = peer_addr;
                    peer_addr.set_port(handshake_result.port);

                    let peer_data = PeerData {
                        endpoint: peer_addr,
                        latest_peer_connection_state: LatestPeerConnectionState::Success,
                        connection: handshake_result.peer_connection.clone(),
                        identity: target_p2p_key.clone(),
                    };
                    let peer_data = Arc::new(peer_data);
                    peers.lock().await.insert(target_p2p_key, peer_data.clone());
                }
            });
        }
    }
    async fn connect_to(
        &self,
        addr: SocketAddr,
        target_p2p_key: ed25519::PublicKey,
    ) -> tokio::io::Result<ConnectionResult> {
        info!("connecting to  {:#?}, p2p_key: {:#?}", addr, target_p2p_key);
        // Check if we have a live connection
        {
            let peers = self.peers.lock().await;
            if let Some(_peer) = peers.get(&target_p2p_key) {
                info!(
                    "already connected to peer returning,  {:#?}, p2p_key: {:#?}, _peer {:#?}",
                    addr, target_p2p_key, _peer
                );
                return Err(Error::new(std::io::ErrorKind::Other, "already connected to peer"));
            }
        }
        // Need to create new connection
        let socket = TcpStream::connect(addr).await?;
        let peer_connection = PeerConnection::start_connection_from_active(
            socket,
            self.p2p_data.message_tx.clone(),
            self.p2p_data.p2p_key.clone(),
            target_p2p_key.clone(),
            addr,
            self.p2p_data.port,
        )
        .await?;

        if let ConnectionResult::Successful(handshake_result) = peer_connection {
            let peer_data = PeerData {
                endpoint: addr,
                latest_peer_connection_state: LatestPeerConnectionState::Success,
                connection: handshake_result.peer_connection.clone(),
                identity: target_p2p_key.clone(),
            };
            let peer_data = Arc::new(peer_data);
            self.peers.lock().await.insert(target_p2p_key, peer_data.clone());

            return Ok(ConnectionResult::Successful(handshake_result));
        } else {
            return Ok(ConnectionResult::InvalidHandshake);
        }
    }

    pub async fn get_random_peer(&self) -> Option<Arc<PeerData>> {
        let peers = self.peers.lock().await;
        let peer = peers.values().choose(&mut OsRng);
        if let Some(peer) = peer {
            return Some(peer.clone());
        } else {
            return None;
        }
    }
    pub async fn get_all_peers(
        peers: Arc<Mutex<HashMap<ed25519::PublicKey, Arc<PeerData>>>>,
    ) -> Vec<Arc<PeerConnection>> {
        let mut peer_connections = vec![];
        for peer in peers.lock().await.values() {
            peer_connections.push(peer.connection.clone());
        }
        return peer_connections;
    }

    async fn run_peer_handler(&self, initial_peers: Vec<NodeRecord>) {
        info!("Starting peer handler loop with initial peers of {:#?}", initial_peers);
        let mut previously_known_peers: HashMap<ed25519::PublicKey, NodeRecord> = HashMap::new();

        for peer in initial_peers {
            previously_known_peers.insert(
                peer.p2p_key.clone(),
                NodeRecord {
                    p2p_key: peer.p2p_key,
                    endpoint: peer.endpoint,
                    state: peer.state,
                    session_state: PeerSessionState::None,
                },
            );
        }
        let peers = self.peers.clone();
        let p2p_data = self.p2p_data.clone();
        self.peer_handler_thread(&mut previously_known_peers, p2p_data, peers).await;
    }
    async fn peer_handler_thread(
        &self,
        previously_known_peers: &mut HashMap<ed25519::PublicKey, NodeRecord>,
        p2p_data: Arc<P2PData>,
        peers: Arc<Mutex<HashMap<ed25519::PublicKey, Arc<PeerData>>>>,
    ) {
        loop {
            self.add_peers(previously_known_peers, peers.clone(), p2p_data.clone()).await;
            self.connect_to_new_peers(previously_known_peers, peers.clone()).await;
            sleep(Duration::from_secs(1)).await;
        }
    }
    async fn add_peers(
        &self,
        previously_known_peers: &mut HashMap<ed25519::PublicKey, NodeRecord>,
        peers: Arc<Mutex<HashMap<ed25519::PublicKey, Arc<PeerData>>>>,
        p2p_data: Arc<P2PData>,
    ) {
        let peer_reponse = self.request_peers().await;
        P2PDaemon::sync_peer_state_to_record(previously_known_peers, peers.clone()).await;
        P2PDaemon::clean_up_disconnected_peers(previously_known_peers, peers.clone()).await;

        let Ok(peers) = peer_reponse else { return };

        for peer in peers {
            let peer_is_self = p2p_data.p2p_key.public_key() == peer.p2p_key;
            //do not add self to peer record
            if peer_is_self {
                continue;
            }
            //if already exists then will only overwrite if last time successfully connected > 12 hours
            let already_exists = previously_known_peers.contains_key(&peer.p2p_key);
            if already_exists {
                let existing_peer = previously_known_peers.get(&peer.p2p_key).unwrap();

                if let PeerHistory::LastConnectionTime(timestamp) = existing_peer.state {
                    let more_then_24_hours_since_last_connect =
                        timestamp.elapsed() > Duration::from_secs(86400);
                    if more_then_24_hours_since_last_connect {
                        //overwrite
                        let node_record = NodeRecord {
                            p2p_key: peer.p2p_key,
                            endpoint: peer.endpoint,
                            state: PeerHistory::NeverConnectedAddedAt(Instant::now()),
                            session_state: PeerSessionState::None,
                        };
                        previously_known_peers.insert(node_record.p2p_key.clone(), node_record);
                    }
                } else if let PeerHistory::NeverConnectedAddedAt(_) = existing_peer.state {
                    //never connected
                    //overwrite
                    let node_record = NodeRecord {
                        p2p_key: peer.p2p_key,
                        endpoint: peer.endpoint,
                        state: PeerHistory::NeverConnectedAddedAt(Instant::now()),
                        session_state: PeerSessionState::None,
                    };

                    previously_known_peers.insert(node_record.p2p_key.clone(), node_record);
                }
            } else {
                let node_record = NodeRecord {
                    p2p_key: peer.p2p_key,
                    endpoint: peer.endpoint,
                    state: PeerHistory::NeverConnectedAddedAt(Instant::now()),
                    session_state: PeerSessionState::None,
                };
                previously_known_peers.insert(node_record.p2p_key.clone(), node_record);
            }
        }
    }
    async fn sync_peer_state_to_record(
        previously_known_peers: &mut HashMap<ed25519::PublicKey, NodeRecord>,
        peers: Arc<Mutex<HashMap<ed25519::PublicKey, Arc<PeerData>>>>,
    ) {
        for peer in peers.lock().await.values() {
            if peer.latest_peer_connection_state == LatestPeerConnectionState::Success {
                let node_record = NodeRecord {
                    p2p_key: peer.identity.clone(),
                    endpoint: peer.endpoint,
                    state: PeerHistory::LastConnectionTime(Instant::now()),
                    session_state: PeerSessionState::SuccessfullyConnected,
                };
                previously_known_peers.insert(peer.identity.clone(), node_record);
            }
        }
    }
    async fn clean_up_disconnected_peers(
        previously_known_peers: &mut HashMap<ed25519::PublicKey, NodeRecord>,
        peers: Arc<Mutex<HashMap<ed25519::PublicKey, Arc<PeerData>>>>,
    ) {
        let mut peers = peers.lock().await;

        let mut peer_keys_to_remove = Vec::new();
        for (key, peer) in peers.iter() {
            let state = peer.connection.peer_connection_state.lock().await;
            if *state == PeerConnectionState::DisconnectedCleanUp {
                peer_keys_to_remove.push(key.clone());
            }
        }
        for key in peer_keys_to_remove {
            info!("Cleaning up disconnected peer, p2p_key:  {:#?}", key);
            peers.remove(&key);
            previously_known_peers.entry(key).and_modify(|e| {
                e.session_state = PeerSessionState::UnsuccessfulConnection(Instant::now());
                e.state = PeerHistory::LastConnectionTime(Instant::now());
            });
        }
    }
    async fn connect_to_new_peers(
        &self,
        previously_known_peers: &mut HashMap<ed25519::PublicKey, NodeRecord>,
        peers: Arc<Mutex<HashMap<ed25519::PublicKey, Arc<PeerData>>>>,
    ) {
        let target_peers = 200;
        let current_amount_peers = peers.lock().await.len();
        for peer in previously_known_peers.values_mut() {
            let have_enough_peers = current_amount_peers >= target_peers;
            if have_enough_peers {
                break;
            }
            if peer.session_state == PeerSessionState::None {
                if let Ok(ConnectionResult::Successful(_res)) =
                    self.connect_to(peer.endpoint, peer.p2p_key.clone()).await
                {
                    peer.session_state = PeerSessionState::SuccessfullyConnected;
                } else {
                    peer.session_state = PeerSessionState::UnsuccessfulConnection(Instant::now());
                }
            } else if let PeerSessionState::UnsuccessfulConnection(time) = peer.session_state {
                let should_try_another = time.elapsed() > Duration::from_secs(10);
                if should_try_another {
                    if let Ok(ConnectionResult::Successful(_res)) =
                        self.connect_to(peer.endpoint, peer.p2p_key.clone()).await
                    {
                        peer.session_state = PeerSessionState::SuccessfullyConnected;
                    } else {
                        peer.session_state =
                            PeerSessionState::UnsuccessfulConnection(Instant::now());
                    }
                }
            } else if peer.session_state == PeerSessionState::SuccessfullyConnected {
                //if successfully connected then do nothing
                //in shutdown thread should change this enum when cleaning up disconnected peers
            }
        }
    }
    async fn request_peers(&self) -> Result<Vec<GetPeersEntry>, ()> {
        let peer_request = Payload { payload_type: PayloadType::GetPeersRequest, content: vec![] };
        let payload = peer_request.encode();
        let response = self.send_request_to_random_peer(payload).await;
        if let Ok(message) = response {
            let content = message.payload;

            if let Ok(payload) = Payload::decode(&content) {
                let get_peers_reply = GetPeersReply::decode(&payload.content);

                if let Ok(get_peers_reply) = get_peers_reply {
                    return Ok(get_peers_reply.peers);
                }
            }
        }
        return Err(());
    }
    pub async fn get_all_peers_clone(&self) -> Vec<GetPeersEntry> {
        let peers = self.peers.lock().await.clone();

        let mut peer_entries = vec![];
        for peer in peers.values() {
            let peer_entry =
                GetPeersEntry { p2p_key: peer.identity.clone(), endpoint: peer.endpoint };
            peer_entries.push(peer_entry);
        }
        return peer_entries;
    }
    pub async fn handle_peer_discovery_request(
        &self,
        peer_connection: Arc<PeerConnection>,
        response_id: u64,
    ) {
        let peer_reply = self.get_all_peers_clone().await;

        let get_peers_reply = GetPeersReply { peers: peer_reply };
        let content = get_peers_reply.encode();

        let peer_request_response = Payload { payload_type: PayloadType::GetPeersReply, content };
        let payload = peer_request_response.encode();

        let _ = peer_connection.send_response(payload.to_vec(), response_id).await;
    }

    pub fn new(
        message_request_tx: UnboundedSender<ReceivedMessage>,
        p2p_key: ed25519::PrivateKey,
        port: u16,
    ) -> P2PDaemon {
        return P2PDaemon {
            peers: Arc::new(Mutex::new(HashMap::new())),
            p2p_data: Arc::new(P2PData {
                message_tx: message_request_tx,
                p2p_key: p2p_key,
                port: port,
            }),
        };
    }
}

#[derive(Debug)]
pub struct P2PDaemon {
    peers: Arc<Mutex<HashMap<ed25519::PublicKey, Arc<PeerData>>>>,
    p2p_data: Arc<P2PData>,
}
#[derive(Debug)]
struct P2PData {
    message_tx: UnboundedSender<ReceivedMessage>,
    p2p_key: ed25519::PrivateKey,
    port: u16,
}

#[derive(Clone, PartialEq, Debug)]
pub enum PeerConnectionState {
    Connected,
    DisconnectedCleanUp,
}

#[derive(Clone, PartialEq, Debug)]
pub enum PeerHistory {
    LastConnectionTime(Instant),
    NeverConnectedAddedAt(Instant),
    BootstrappingNode,
}
#[derive(Clone, PartialEq, Debug)]
pub enum PeerSessionState {
    SuccessfullyConnected,
    UnsuccessfulConnection(Instant),
    None,
}
#[derive(Clone, PartialEq, Debug)]
pub struct NodeRecord {
    pub p2p_key: ed25519::PublicKey,
    pub endpoint: SocketAddr,
    pub state: PeerHistory,
    pub session_state: PeerSessionState, //since starting node
}
#[derive(Clone, PartialEq, Debug)]
pub enum LatestPeerConnectionState {
    Success,
    FailedToConnect,
}

#[derive(Debug)]
pub struct PeerData {
    identity: ed25519::PublicKey,
    endpoint: SocketAddr,
    latest_peer_connection_state: LatestPeerConnectionState,
    connection: Arc<PeerConnection>,
}

#[derive(Clone, PartialEq, Debug)]
pub struct SuccessfulHandshake {
    pub p2p_key: ed25519::PublicKey,
    pub port: u16,
}
#[derive(Clone, PartialEq, Debug)]
pub enum HandShakeResults {
    SuccessfulHandshake(SuccessfulHandshake),
    InvalidPing,
    FirstMessageReceivedWasNotPing,
}

use crate::p2p::peer_connection::{ConnectionResult, PeerConnection};
use crate::p2p::types::getpeers::{GetPeersEntry, GetPeersReply};
use crate::p2p::types::messages::ReceivedMessage;
use crate::p2p::types::messages::{Message, MessageType};
use crate::p2p::types::payload::{Payload, PayloadType};
use rand::rngs::OsRng;
use rand::seq::IteratorRandom;
use shared_types::borsh::BorshExt;
use shared_types::crypto::ed25519;
use std::io::Error;
use std::sync::Arc;
use std::{collections::HashMap, net::SocketAddr, time::Duration};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use tokio::time::Instant;
use tokio::{sync::mpsc::UnboundedSender, time::sleep};
use tracing::info;
