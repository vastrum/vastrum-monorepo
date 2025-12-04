impl PeerConnection {
    pub async fn start_connection_from_active(
        socket: TcpStream,
        message_request_tx: UnboundedSender<ReceivedMessage>,
        p2p_key_of_this_node: ed25519::PrivateKey,
        p2p_key_of_remote: ed25519::PublicKey,
        addr: SocketAddr,
        local_listening_port: u16,
    ) -> tokio::io::Result<ConnectionResult> {
        let peer_connection = PeerConnection::new(socket, p2p_key_of_this_node);

        let handshake_result = PeerConnection::execute_active_handshake(
            peer_connection.clone(),
            p2p_key_of_remote.clone(),
            local_listening_port,
        )
        .await?;

        if let HandShakeResults::SuccessfulHandshake(_) = handshake_result {
            PeerConnection::start_server_and_pinger(
                peer_connection.clone(),
                message_request_tx,
                p2p_key_of_remote.clone(),
            )
            .await;

            return Ok(ConnectionResult::Successful(SuccessfulConnectionResult {
                peer_connection: peer_connection,
                p2p_key: p2p_key_of_remote,
                port: addr.port(),
            }));
        } else {
            //socket will get auto cleaned up here?
            return Ok(ConnectionResult::InvalidHandshake);
        }
    }
    async fn execute_active_handshake(
        peer_connection: Arc<PeerConnection>,
        p2p_key_of_remote: ed25519::PublicKey,
        local_listening_port: u16,
    ) -> tokio::io::Result<HandShakeResults> {
        let ping_hash = peer_connection.send_ping(p2p_key_of_remote, local_listening_port).await?;

        let message = PeerConnection::read_next_message(peer_connection.clone()).await?;
        if let MessageType::Pong = message.message_type {
            let pong = PongMessage::decode(&message.payload).unwrap();

            let local_node_p2p_key = peer_connection.local_p2p_key.public_key();
            let content_hash = pong.content.calculate_hash();
            let content = pong.content;

            let current_unix_timestamp =
                SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
            let time_since_pong_sec = current_unix_timestamp - content.unix_timestamp;

            let valid_signature =
                content.from_p2p_key.verify_signature_hash(content_hash, &pong.signature);
            let valid_timestamp = time_since_pong_sec < 30;
            let valid_dest = content.to_p2p_key == local_node_p2p_key;
            let valid_hash = content.ping_message_hash == ping_hash;
            let is_valid_handshake = valid_signature && valid_timestamp && valid_dest && valid_hash;
            if is_valid_handshake {
                return Ok(HandShakeResults::SuccessfulHandshake(SuccessfulHandshake {
                    p2p_key: content.from_p2p_key,
                    port: content.listening_port,
                }));
            } else {
                return Ok(HandShakeResults::InvalidPing);
            }
        } else {
            return Ok(HandShakeResults::FirstMessageReceivedWasNotPing); //not pong
        }
    }
    pub async fn start_connection_from_listener(
        socket: TcpStream,
        message_request_tx: UnboundedSender<ReceivedMessage>,
        p2p_key_of_this_node: ed25519::PrivateKey,
        local_listening_port: u16,
    ) -> tokio::io::Result<ConnectionResult> {
        let peer_connection = PeerConnection::new(socket, p2p_key_of_this_node);

        let handshake_result = PeerConnection::execute_passive_handshake(
            peer_connection.clone(),
            local_listening_port,
        )
        .await?;

        if let HandShakeResults::SuccessfulHandshake(handshake_result) = handshake_result {
            PeerConnection::start_server_and_pinger(
                peer_connection.clone(),
                message_request_tx,
                handshake_result.p2p_key.clone(),
            )
            .await;
            return Ok(ConnectionResult::Successful(SuccessfulConnectionResult {
                peer_connection: peer_connection,
                p2p_key: handshake_result.p2p_key,
                port: handshake_result.port,
            }));
        } else {
            return Ok(ConnectionResult::InvalidHandshake);
        }
    }
    async fn execute_passive_handshake(
        peer_connection: Arc<PeerConnection>,
        local_listening_port: u16,
    ) -> tokio::io::Result<HandShakeResults> {
        let message = PeerConnection::read_next_message(peer_connection.clone()).await?;
        if let MessageType::Ping = message.message_type {
            let ping = PingMessage::decode(&message.payload).unwrap();

            let local_node_p2p_key = peer_connection.local_p2p_key.public_key();
            let content_hash = ping.content.calculate_hash();
            let content = ping.content;

            let current_unix_timestamp =
                SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
            let time_since_ping_sec = current_unix_timestamp - content.unix_timestamp;

            let valid_signature =
                content.from_p2p_key.verify_signature_hash(content_hash, &ping.signature);
            let valid_timestamp = time_since_ping_sec < 30;
            let valid_dest = content.to_p2p_key == local_node_p2p_key;

            let is_valid_handshake = valid_signature && valid_timestamp && valid_dest;
            if is_valid_handshake {
                peer_connection
                    .send_pong(content.from_p2p_key.clone(), content_hash, local_listening_port)
                    .await?;
                return Ok(HandShakeResults::SuccessfulHandshake(SuccessfulHandshake {
                    p2p_key: content.from_p2p_key,
                    port: content.listening_port,
                }));
            } else {
                return Ok(HandShakeResults::InvalidPing);
            }
        } else {
            return Ok(HandShakeResults::FirstMessageReceivedWasNotPing);
        }
    }
    async fn send_pong(
        &self,
        to_pub_key: ed25519::PublicKey,
        ping_hash: Sha256Digest,
        local_listening_port: u16,
    ) -> tokio::io::Result<()> {
        //send pong response
        let current_unix_timestamp =
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        let content = PongContent {
            from_p2p_key: self.local_p2p_key.public_key(),
            unix_timestamp: current_unix_timestamp,
            to_p2p_key: to_pub_key,
            listening_port: local_listening_port,
            ping_message_hash: ping_hash,
        };
        let content_hash = content.calculate_hash();
        let signature = self.local_p2p_key.sign_hash(content_hash);
        let pong = PongMessage { signature: signature, content: content };
        let message = Message { id: 0, message_type: MessageType::Pong, payload: pong.encode() };
        self.send_message(&message).await?;
        return Ok(());
    }
    async fn send_ping(
        &self,
        to_pub_key: ed25519::PublicKey,
        local_listening_port: u16,
    ) -> tokio::io::Result<Sha256Digest> {
        //send pong response
        let current_unix_timestamp =
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        let content = PingContent {
            from_p2p_key: self.local_p2p_key.public_key(),
            unix_timestamp: current_unix_timestamp,
            to_p2p_key: to_pub_key,
            listening_port: local_listening_port,
        };
        let content_hash = content.calculate_hash();
        let signature = self.local_p2p_key.sign_hash(content_hash);
        let ping = PingMessage { signature: signature, content: content };
        let message = Message { id: 0, message_type: MessageType::Ping, payload: ping.encode() };
        self.send_message(&message).await?;
        return Ok(content_hash);
    }
    async fn read_next_message(peer_connection: Arc<PeerConnection>) -> tokio::io::Result<Message> {
        let mut reader = peer_connection.reader.lock().await;

        // Read message length (4 bytes)
        let mut len_bytes = [0u8; 4];
        //in case of eof exception returned here opposite side disconnected?
        reader.read_exact(&mut len_bytes).await?;

        let len = u32::from_be_bytes(len_bytes) as usize;
        let message_too_big = len > 50 * 1024 * 1024; //Todo lower this
        if message_too_big {
            return Err(std::io::ErrorKind::OutOfMemory.into());
        }
        // Read message body
        let mut buffer = vec![0u8; len];
        reader.read_exact(&mut buffer).await?;
        let message = Message::decode(&mut Bytes::from(buffer)).unwrap();
        *peer_connection.last_receive.lock().await = Instant::now();
        return Ok(message);
    }
    async fn start_server_and_pinger(
        peer_connection: Arc<PeerConnection>,
        message_request_tx: UnboundedSender<ReceivedMessage>,
        remote_p2p_key: ed25519::PublicKey,
    ) {
        let pc = peer_connection.clone();
        let server_thread = tokio::spawn(async move {
            let _res =
                PeerConnection::server_listener(peer_connection.clone(), message_request_tx).await;
            //if here then socket has been closed
            peer_connection.cleanup().await;
        });
        *pc.server_thread.lock().await = server_thread;

        let pc2 = pc.clone();
        let pinger_thread = tokio::spawn(async move {
            sleep(Duration::from_secs(20)).await;
            PeerConnection::pinger_heartbeat(pc.clone(), remote_p2p_key.clone()).await;
        });
        *pc2.pinger_thread.lock().await = pinger_thread;
    }
    async fn server_listener(
        peer_connection: Arc<PeerConnection>,
        message_request_tx: UnboundedSender<ReceivedMessage>,
    ) -> tokio::io::Result<()> {
        loop {
            let message = PeerConnection::read_next_message(peer_connection.clone()).await?;

            if let MessageType::Request = message.message_type {
                message_request_tx
                    .send(ReceivedMessage {
                        message: message,
                        from_peer_connection: peer_connection.clone(),
                    })
                    .unwrap();
            } else if let MessageType::Response = message.message_type {
                let mut pending = peer_connection.pending_requests.lock().await;
                if let Some(sender) = pending.remove(&message.id) {
                    let _ = sender.send(message);
                }
            } else if let MessageType::Statement = message.message_type {
                message_request_tx
                    .send(ReceivedMessage {
                        message: message,
                        from_peer_connection: peer_connection.clone(),
                    })
                    .unwrap();
            } else if let MessageType::Ping = message.message_type {
                PeerConnection::process_pinger_heartbeat(peer_connection.clone(), message.payload)
                    .await;
            }
        }
    }
    pub async fn send_message(&self, message: &Message) -> tokio::io::Result<()> {
        let msg_bytes = message.encode();

        let len = (msg_bytes.len() as u32).to_be_bytes();

        let mut writer = self.writer.lock().await;
        writer.write_all(&len).await?;
        writer.write_all(&msg_bytes).await?;
        Ok(())
    }
    pub async fn send_request(&self, payload: Vec<u8>) -> tokio::io::Result<Message> {
        let request_id;
        {
            let mut next_id = self.next_request_id.lock().await;
            request_id = *next_id;
            *next_id += 1;
        }

        let (tx, rx) = oneshot::channel();

        {
            let mut pending = self.pending_requests.lock().await;
            pending.insert(request_id, tx);
        }

        let request = Message { id: request_id, message_type: MessageType::Request, payload };

        self.send_message(&request).await?;

        match tokio::time::timeout(std::time::Duration::from_secs(10), rx).await {
            Ok(Ok(response)) => Ok(response),
            Ok(Err(_)) => {
                Err(tokio::io::Error::new(tokio::io::ErrorKind::Other, "Response channel closed"))
            }
            Err(_) => {
                self.pending_requests.lock().await.remove(&request_id);
                Err(tokio::io::Error::new(tokio::io::ErrorKind::TimedOut, "Request timed out"))
            }
        }
    }
    pub async fn send_response(&self, payload: Vec<u8>, response_id: u64) -> tokio::io::Result<()> {
        let response = Message { id: response_id, message_type: MessageType::Response, payload };
        self.send_message(&response).await?;

        return Ok(());
    }
    async fn pinger_heartbeat(
        peer_connection: Arc<PeerConnection>,
        _remote_p2p_key: ed25519::PublicKey,
    ) {
        let timeout_limit = Duration::from_secs(60);
        loop {
            //let _res = peer_connection.send_ping(remote_p2p_key.clone(), 0).await;
            sleep(Duration::from_secs(5)).await;
            let timeout = peer_connection.last_receive.lock().await.elapsed() > timeout_limit;
            if timeout {
                peer_connection.cleanup().await;
            }
        }
    }
    async fn process_pinger_heartbeat(
        peer_connection: Arc<PeerConnection>,
        message_payload: Vec<u8>,
    ) {
        let ping = PingMessage::decode(&message_payload).unwrap();

        let local_node_p2p_key = peer_connection.local_p2p_key.public_key();
        let content_hash = ping.content.calculate_hash();
        let content = ping.content;

        let current_unix_timestamp =
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        let time_since_ping_sec = current_unix_timestamp - content.unix_timestamp;

        let valid_signature =
            content.from_p2p_key.verify_signature_hash(content_hash, &ping.signature);
        let valid_timestamp = time_since_ping_sec < 30;
        let valid_dest = content.to_p2p_key == local_node_p2p_key;

        let is_valid_ping = valid_signature && valid_timestamp && valid_dest;
        if is_valid_ping {
            //update last receive time
            *peer_connection.last_receive.lock().await = Instant::now();
        }
    }
    pub async fn cleanup(&self) {
        self.server_thread.lock().await.abort();
        self.pinger_thread.lock().await.abort();
        *self.peer_connection_state.lock().await = PeerConnectionState::DisconnectedCleanUp;
    }
    fn new(socket: TcpStream, local_p2p_key: ed25519::PrivateKey) -> Arc<PeerConnection> {
        let (reader, writer) = tokio::io::split(socket);

        let peer_connection = PeerConnection {
            pending_requests: Arc::new(Mutex::new(HashMap::new())),
            last_receive: Arc::new(Mutex::new(Instant::now())),
            writer: Arc::new(Mutex::new(writer)),
            reader: Arc::new(Mutex::new(reader)),
            next_request_id: Arc::new(Mutex::new(0)),
            local_p2p_key: Arc::new(local_p2p_key),
            peer_connection_state: Mutex::new(PeerConnectionState::Connected),
            server_thread: Mutex::new(tokio::spawn(async move {})),
            pinger_thread: Mutex::new(tokio::spawn(async move {})),
        };

        return Arc::new(peer_connection);
    }
}

#[derive(Debug)]
pub struct PeerConnection {
    reader: Arc<Mutex<tokio::io::ReadHalf<TcpStream>>>,
    writer: Arc<Mutex<tokio::io::WriteHalf<TcpStream>>>,
    pending_requests: Arc<Mutex<HashMap<u64, oneshot::Sender<Message>>>>,
    last_receive: Arc<Mutex<Instant>>,
    next_request_id: Arc<Mutex<u64>>,
    local_p2p_key: Arc<ed25519::PrivateKey>,
    pub peer_connection_state: Mutex<PeerConnectionState>,
    server_thread: Mutex<JoinHandle<()>>,
    pinger_thread: Mutex<JoinHandle<()>>,
}

pub struct SuccessfulConnectionResult {
    pub peer_connection: Arc<PeerConnection>,
    pub p2p_key: ed25519::PublicKey,
    pub port: u16,
}
pub enum ConnectionResult {
    Successful(SuccessfulConnectionResult),
    InvalidHandshake,
}

use crate::p2p::domon::{HandShakeResults, PeerConnectionState, SuccessfulHandshake};
use crate::p2p::types::messages::ReceivedMessage;
use crate::p2p::types::messages::{
    Message, MessageType, PingContent, PingMessage, PongContent, PongMessage,
};
use bytes::Bytes;
use shared_types::borsh::BorshExt;
use shared_types::crypto::ed25519;
use shared_types::crypto::sha256::Sha256Digest;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use std::{collections::HashMap, net::SocketAddr, time::Duration};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio::sync::oneshot;
use tokio::task::JoinHandle;
use tokio::time::Instant;
use tokio::{sync::mpsc::UnboundedSender, time::sleep};
