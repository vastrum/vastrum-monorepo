use crate::utils::limits::{
    MAX_INBOUND_NORMAL, MAX_INBOUND_VALIDATORS, MAX_OUTBOUND_NORMAL, MAX_OUTBOUND_VALIDATORS,
    MAX_PEER_RECORDS,
};

pub struct PeerManager {
    peers: Arc<Mutex<BTreeMap<ed25519::PublicKey, PeerRecord>>>,
    connection_inbound_tx: mpsc::UnboundedSender<InboundMessage>,
    local_p2p_key: ed25519::PrivateKey,
    handshake_rate_limiter: Mutex<HandshakeRateLimiter>,
    release_tx: mpsc::UnboundedSender<ed25519::PublicKey>,
    validator_p2p_keys: HashSet<ed25519::PublicKey>,
}
impl PeerManager {
    pub async fn start(
        app_inbound_tx: mpsc::UnboundedSender<AppInboundMessage>,
        local_p2p_key: ed25519::PrivateKey,
        initial_peers: Vec<KnownPeer>,
        validator_p2p_keys: HashSet<ed25519::PublicKey>,
    ) -> Arc<PeerManager> {
        let (connection_inbound_tx, connection_inbound_rx) = mpsc::unbounded_channel();
        let (release_tx, release_rx) = mpsc::unbounded_channel();

        let pm = Arc::new(PeerManager {
            peers: Arc::new(Mutex::new(BTreeMap::new())),
            connection_inbound_tx,
            local_p2p_key,
            handshake_rate_limiter: Mutex::new(HandshakeRateLimiter::new()),
            release_tx,
            validator_p2p_keys,
        });

        pm.add_known_peers(initial_peers, EndpointSource::Bootstrap);

        pm.start_listener();
        pm.start_discovery_loop();
        pm.start_message_router(connection_inbound_rx, app_inbound_tx);

        pm.start_peer_handler(release_rx);

        return pm;
    }

    fn start_listener(self: &Arc<Self>) {
        let pm = self.clone();
        tokio::spawn(async move {
            let bind_addr =
                SocketAddr::from((std::net::Ipv4Addr::UNSPECIFIED, vastrum_shared_types::ports::P2P_PORT));
            let listener =
                TcpListener::bind(bind_addr).await.expect("failed to bind listener port");
            loop {
                let Ok((socket, addr)) = listener.accept().await else {
                    continue;
                };
                let pm = pm.clone();
                tokio::spawn(async move {
                    let Some(_permit) =
                        pm.handshake_rate_limiter.lock().can_accept_handshake(addr.ip())
                    else {
                        return;
                    };
                    let Ok(result) =
                        handshake_listen(socket, &pm.local_p2p_key, vastrum_shared_types::ports::P2P_PORT)
                            .await
                    else {
                        return;
                    };

                    let tier = pm.get_peer_tier(&result.remote_p2p_key);

                    let mut peers = pm.peers.lock();
                    if let Some(record) = peers.get(&result.remote_p2p_key) {
                        let already_connected = record.connection_status.is_connected();
                        if already_connected {
                            return;
                        }

                        let simultaneous_connections_tie_breaker =
                            record.connection_status.is_connecting()
                                && result.remote_p2p_key < pm.local_p2p_key.public_key();
                        if simultaneous_connections_tie_breaker {
                            return;
                        }
                    }

                    match tier {
                        PeerTier::Validator => {
                            let free_inbound_validator_connections = MAX_INBOUND_VALIDATORS
                                .saturating_sub(peers.inbound_validator_connections());

                            if free_inbound_validator_connections == 0 {
                                return;
                            }
                        }
                        PeerTier::Normal => {
                            let free_inbound_normal_connections = MAX_INBOUND_NORMAL
                                .saturating_sub(peers.inbound_normal_connections());

                            if free_inbound_normal_connections == 0 {
                                return;
                            }
                        }
                    }

                    let sender = spawn_connection_actor(
                        result.encrypted_reader,
                        result.encrypted_writer,
                        result.remote_p2p_key,
                        pm.connection_inbound_tx.clone(),
                        pm.release_tx.clone(),
                        rate_config_for_tier(tier),
                    );

                    peers.insert(
                        result.remote_p2p_key,
                        PeerRecord::new(
                            result.remote_p2p_key,
                            result.remote_endpoint,
                            ConnectionStatus::Connected(sender, ConnectionOrigin::Inbound),
                            tier,
                            EndpointSource::Discovery,
                        ),
                    );
                });
            }
        });
    }

    async fn connect_to(&self, addr: SocketAddr, target_p2p_key: ed25519::PublicKey) {
        let tier = self.get_peer_tier(&target_p2p_key);
        {
            let mut peers = self.peers.lock();
            let record = peers.entry(target_p2p_key).or_insert_with(|| {
                PeerRecord::new(
                    target_p2p_key,
                    addr,
                    ConnectionStatus::Idle,
                    tier,
                    EndpointSource::Discovery,
                )
            });

            if record.connection_status.is_connected()
                || record.connection_status.is_connecting()
                || record.connection_status.in_backoff()
            {
                return;
            }
            record.connection_status = ConnectionStatus::Connecting;
        }

        let res = handshake_dial(
            addr,
            &self.local_p2p_key,
            target_p2p_key,
            vastrum_shared_types::ports::P2P_PORT,
        )
        .await;

        let mut peers = self.peers.lock();
        let Some(record) = peers.get_mut(&target_p2p_key) else { return };

        let already_connected_through_other_path = !record.connection_status.is_connecting();
        if already_connected_through_other_path {
            return;
        }

        match res {
            Ok(handshake) => {
                let sender = spawn_connection_actor(
                    handshake.encrypted_reader,
                    handshake.encrypted_writer,
                    handshake.remote_p2p_key,
                    self.connection_inbound_tx.clone(),
                    self.release_tx.clone(),
                    rate_config_for_tier(tier),
                );
                record.connection_status =
                    ConnectionStatus::Connected(sender, ConnectionOrigin::Outbound);
            }
            Err(..) => {
                record.failed_attempts += 1;
                record.connection_status = ConnectionStatus::Backoff {
                    until: Instant::now() + backoff_duration(record.failed_attempts),
                };
            }
        }
    }

    pub fn broadcast_statement_to_all_peers(self: &Arc<Self>, payload: AppPayload) {
        let pm = self.clone();
        tokio::spawn(async move {
            let encoded = NetworkPayload::App(payload).encode();
            for sender in &pm.connected_peers() {
                let _ = sender.send_statement(encoded.clone());
            }
        });
    }

    fn connected_peers(&self) -> Vec<PeerSender> {
        let peers = self.peers.lock();
        let mut connected = Vec::new();
        for record in peers.values() {
            if let ConnectionStatus::Connected(sender, _) = &record.connection_status {
                connected.push(sender.clone());
            }
        }
        return connected;
    }

    fn random_connected_peer(&self) -> Option<PeerSender> {
        let connected = self.connected_peers();
        let choosen_peer = rng::choose(&connected).cloned();
        return choosen_peer;
    }

    pub async fn send_request_to_random_peer(&self, payload: AppPayload) -> Option<Response> {
        self.send_raw_request_to_random_peer(NetworkPayload::App(payload).encode()).await
    }

    async fn send_raw_request_to_random_peer(&self, payload: Vec<u8>) -> Option<Response> {
        let sender = self.random_connected_peer()?;
        sender.send_request(payload, 3).await.ok()
    }

    fn add_known_peers(&self, new_peers: Vec<KnownPeer>, source: EndpointSource) {
        let local_pub = self.local_public_key();
        let mut peers = self.peers.lock();

        for peer in new_peers.into_iter().take(150) {
            if peers.len() > MAX_PEER_RECORDS {
                return;
            }
            if peer.p2p_key == local_pub {
                continue;
            }
            /*if let Some(existing) = locked.get(&peer.p2p_key) {
                let is_bootstrap_peer = existing.endpoint_source == EndpointSource::Bootstrap;
                //dont update bootstrap peers
                if is_bootstrap_peer {
                    continue;
                }
                let is_currently_connected = existing.connection_status.is_connected()
                    || existing.connection_status.is_connecting();
                if is_currently_connected {
                    continue;
                }
            }*/
            if peers.contains_key(&peer.p2p_key) {
                continue;
            }
            let tier = self.get_peer_tier(&peer.p2p_key);
            peers.insert(
                peer.p2p_key,
                PeerRecord::new(peer.p2p_key, peer.endpoint, ConnectionStatus::Idle, tier, source),
            );
        }
    }
    pub fn local_public_key(&self) -> ed25519::PublicKey {
        self.local_p2p_key.public_key()
    }

    fn get_peer_tier(&self, p2p_key: &ed25519::PublicKey) -> PeerTier {
        if self.validator_p2p_keys.contains(p2p_key) {
            return PeerTier::Validator;
        } else {
            return PeerTier::Normal;
        }
    }

    fn start_peer_handler(
        self: &Arc<Self>,
        mut release_rx: mpsc::UnboundedReceiver<ed25519::PublicKey>,
    ) {
        let pm = self.clone();
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_secs(1)).await;
                while let Ok(peer_key) = release_rx.try_recv() {
                    pm.handle_peer_died(peer_key);
                }
                pm.connect_to_unconnected_peers();
            }
        });
    }

    fn handle_peer_died(&self, peer_key: ed25519::PublicKey) {
        let mut peers = self.peers.lock();
        if let Some(record) = peers.get_mut(&peer_key) {
            if record.connection_status.is_connected() {
                record.failed_attempts = 0;
                let until = Instant::now() + backoff_duration(0);
                record.connection_status = ConnectionStatus::Backoff { until };
            }
        }
    }

    fn start_message_router(
        self: &Arc<Self>,
        mut rx: mpsc::UnboundedReceiver<InboundMessage>,
        app_tx: mpsc::UnboundedSender<AppInboundMessage>,
    ) {
        let pm = self.clone();
        tokio::spawn(async move {
            while let Some(msg) = rx.recv().await {
                let Ok(network_payload) = NetworkPayload::decode(&msg.payload) else {
                    continue;
                };

                match network_payload {
                    NetworkPayload::P2pGetPeersRequest => {
                        pm.handle_get_peers_request(msg.respond);
                    }
                    NetworkPayload::App(app_payload) => {
                        let _ = app_tx
                            .send(AppInboundMessage { payload: app_payload, respond: msg.respond });
                    }
                }
            }
        });
    }

    fn handle_get_peers_request(&self, respond: Option<ResponseHandle>) {
        let locked = self.peers.lock();
        let mut peers = Vec::new();
        for r in locked.values() {
            if r.connection_status.is_connected() {
                peers.push(KnownPeer { p2p_key: r.p2p_key, endpoint: r.endpoint });
            }
        }
        if let Some(respond) = respond {
            respond.respond(GetPeersReply { peers }.encode());
        }
    }
    async fn get_peers(&self) {
        let request = NetworkPayload::P2pGetPeersRequest.encode();
        let Some(response) = self.send_raw_request_to_random_peer(request).await else { return };
        let Ok(reply) = GetPeersReply::decode(&response.payload) else { return };
        self.add_known_peers(reply.peers, EndpointSource::Discovery);
    }
    fn start_discovery_loop(self: &Arc<Self>) {
        let pm = self.clone();
        tokio::spawn(async move {
            loop {
                let pm = pm.clone();
                tokio::spawn(async move { pm.get_peers().await });
                sleep(Duration::from_secs(1)).await;
            }
        });
    }

    fn connect_to_unconnected_peers(self: &Arc<Self>) {
        let peers = self.peers.lock();

        let mut free_outbound_validator_connections =
            MAX_OUTBOUND_VALIDATORS.saturating_sub(peers.outbound_validator_connections());
        let mut free_outbound_normal_connections =
            MAX_OUTBOUND_NORMAL.saturating_sub(peers.outbound_normal_connections());
        for peer in peers.values() {
            let is_idle_candidate = !peer.connection_status.is_connected()
                && !peer.connection_status.is_connecting()
                && !peer.connection_status.in_backoff();

            if is_idle_candidate {
                match peer.tier {
                    PeerTier::Validator => {
                        if free_outbound_validator_connections > 0 {
                            free_outbound_validator_connections -= 1;
                        } else {
                            continue;
                        }
                    }
                    PeerTier::Normal => {
                        if free_outbound_normal_connections > 0 {
                            free_outbound_normal_connections -= 1;
                        } else {
                            continue;
                        }
                    }
                }
                let pm = self.clone();
                let endpoint = peer.endpoint;
                let p2p_key = peer.p2p_key;
                tokio::spawn(async move {
                    pm.connect_to(endpoint, p2p_key).await;
                });
            }
        }
    }
}

pub fn rate_config_for_tier(tier: PeerTier) -> f64 {
    match tier {
        PeerTier::Validator => 10_000.0,
        PeerTier::Normal => 500.0,
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum PeerTier {
    Validator,
    Normal,
}

#[derive(Clone, Copy, PartialEq)]
enum ConnectionOrigin {
    Inbound,
    Outbound,
}

#[derive(Clone, Copy, PartialEq)]
enum EndpointSource {
    Bootstrap,
    Discovery,
}
trait PeerCounts {
    fn inbound_validator_connections(&self) -> usize;
    fn outbound_validator_connections(&self) -> usize;
    fn inbound_normal_connections(&self) -> usize;
    fn outbound_normal_connections(&self) -> usize;
}

impl PeerCounts for BTreeMap<ed25519::PublicKey, PeerRecord> {
    fn inbound_validator_connections(&self) -> usize {
        self.values()
            .filter(|p| p.tier == PeerTier::Validator && p.connection_status.is_inbound())
            .count()
    }
    fn outbound_validator_connections(&self) -> usize {
        self.values()
            .filter(|p| p.tier == PeerTier::Validator && p.connection_status.is_outbound())
            .count()
    }
    fn inbound_normal_connections(&self) -> usize {
        self.values()
            .filter(|p| p.tier == PeerTier::Normal && p.connection_status.is_inbound())
            .count()
    }
    fn outbound_normal_connections(&self) -> usize {
        self.values()
            .filter(|p| p.tier == PeerTier::Normal && p.connection_status.is_outbound())
            .count()
    }
}

enum ConnectionStatus {
    Idle,
    Connecting,
    Connected(PeerSender, ConnectionOrigin),
    Backoff { until: Instant },
}
impl ConnectionStatus {
    fn is_connected(&self) -> bool {
        matches!(self, ConnectionStatus::Connected(..))
    }
    fn is_connecting(&self) -> bool {
        matches!(self, ConnectionStatus::Connecting)
    }
    fn in_backoff(&self) -> bool {
        matches!(self, ConnectionStatus::Backoff { until } if Instant::now() < *until)
    }
    fn is_inbound(&self) -> bool {
        let is_inbound = matches!(self, ConnectionStatus::Connected(_, ConnectionOrigin::Inbound));
        return is_inbound;
    }

    fn is_outbound(&self) -> bool {
        let is_outbound =
            matches!(self, ConnectionStatus::Connected(_, ConnectionOrigin::Outbound))
                || matches!(self, ConnectionStatus::Connecting);
        return is_outbound;
    }
}

struct PeerRecord {
    p2p_key: ed25519::PublicKey,
    endpoint: SocketAddr,
    connection_status: ConnectionStatus,
    failed_attempts: u32,
    tier: PeerTier,
    #[allow(dead_code)]
    endpoint_source: EndpointSource,
}

impl PeerRecord {
    fn new(
        p2p_key: ed25519::PublicKey,
        endpoint: SocketAddr,
        connection_status: ConnectionStatus,
        tier: PeerTier,
        endpoint_source: EndpointSource,
    ) -> Self {
        Self { p2p_key, endpoint, connection_status, failed_attempts: 0, tier, endpoint_source }
    }
}

#[derive(BorshDeserialize, BorshSerialize, Clone, Copy, Debug, PartialEq)]
pub struct KnownPeer {
    pub p2p_key: ed25519::PublicKey,
    pub endpoint: SocketAddr,
}

const BACKOFF_BASE: Duration = Duration::from_secs(2);
const BACKOFF_CAP: Duration = Duration::from_secs(15);

fn backoff_duration(attempts: u32) -> Duration {
    let base = BACKOFF_BASE * 2u32.saturating_pow(attempts.saturating_sub(1));
    let base = base.min(BACKOFF_CAP);
    let max_jitter = base.as_millis() as u64 / 2;
    let jitter_ms = rng::random_range(0..=max_jitter);
    let jitter = Duration::from_millis(jitter_ms);
    (base + jitter).min(BACKOFF_CAP)
}
use crate::p2p::connection::{PeerSender, ResponseHandle, spawn_connection_actor};
use crate::p2p::handshake::{handshake_dial, handshake_listen};
use crate::p2p::handshake_rate_limiter::HandshakeRateLimiter;
use crate::p2p::types::app_types::GetPeersReply;
use crate::p2p::types::messages::{AppInboundMessage, InboundMessage, Response};
use crate::p2p::types::payload::{AppPayload, NetworkPayload};
use crate::utils::rng;
use borsh::{BorshDeserialize, BorshSerialize};
use parking_lot::Mutex;
use vastrum_shared_types::borsh::BorshExt;
use vastrum_shared_types::crypto::ed25519;
use std::collections::{BTreeMap, HashSet};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::sync::mpsc;
use tokio::time::{Instant, sleep};
