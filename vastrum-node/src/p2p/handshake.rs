pub async fn handshake_dial(
    addr: SocketAddr,
    local_key: &ed25519::PrivateKey,
    remote_key: ed25519::PublicKey,
    local_port: u16,
) -> eyre::Result<HandshakeResult> {
    tokio::time::timeout(Duration::from_secs(15), async {
        let socket = TcpStream::connect(addr).await?;
        socket.set_nodelay(true).ok();
        let (reader, writer) = tokio::io::split(socket);
        let mut reader = FramedReader::new(reader);
        let mut writer = FramedWriter::new(writer);
        let ephemeral_private = generate_ephemeral_x25519_key();
        let ephemeral_public = ephemeral_private.public_key();

        let content = HandshakeRequestContent {
            from_p2p_key: local_key.public_key(),
            unix_timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            to_p2p_key: remote_key,
            listening_port: local_port,
            x25519_public_key: ephemeral_public,
        };
        let content_hash = content.calculate_hash();
        let signature = local_key.sign_hash(content_hash);
        let request = HandshakeRequest { signature, content };
        writer.write_frame(&request.encode()).await?;

        let frame = reader.read_frame().await?;
        let response = HandshakeResponse::decode(&Bytes::from(frame))?;
        let response_content_hash = response.content.calculate_hash();

        validate_signature_and_timestamp(
            remote_key,
            response_content_hash,
            response.signature,
            response.content.unix_timestamp,
        )?;
        if response.content.request_hash != content_hash {
            return Err(eyre::eyre!("invalid request hash"));
        }

        let (encrypted_reader, encrypted_writer) = derive_encrypted_transport(
            ephemeral_private,
            response.content.x25519_public_key,
            reader,
            writer,
            true,
        );

        let remote_endpoint = SocketAddr::new(addr.ip(), response.content.listening_port);
        Ok(HandshakeResult {
            encrypted_reader,
            encrypted_writer,
            remote_p2p_key: remote_key,
            remote_endpoint,
        })
    })
    .await?
}

pub async fn handshake_listen(
    socket: TcpStream,
    local_key: &ed25519::PrivateKey,
    local_port: u16,
) -> eyre::Result<HandshakeResult> {
    tokio::time::timeout(Duration::from_secs(10), async {
        socket.set_nodelay(true).ok();
        let peer_ip = socket.peer_addr()?.ip();
        let (reader, writer) = tokio::io::split(socket);
        let mut reader = FramedReader::new(reader);
        let mut writer = FramedWriter::new(writer);
        let ephemeral_private = generate_ephemeral_x25519_key();
        let ephemeral_public = ephemeral_private.public_key();

        let frame = reader.read_frame().await?;
        let request = HandshakeRequest::decode(&Bytes::from(frame))?;
        let request_content_hash = request.content.calculate_hash();

        if request.content.to_p2p_key != local_key.public_key() {
            return Err(eyre::eyre!("invalid to_p2p_key"));
        }
        validate_signature_and_timestamp(
            request.content.from_p2p_key,
            request_content_hash,
            request.signature,
            request.content.unix_timestamp,
        )?;

        let response_content = HandshakeResponseContent {
            unix_timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            listening_port: local_port,
            request_hash: request_content_hash,
            x25519_public_key: ephemeral_public,
        };
        let response_content_hash = response_content.calculate_hash();
        let signature = local_key.sign_hash(response_content_hash);
        let response = HandshakeResponse { signature, content: response_content };
        writer.write_frame(&response.encode()).await?;

        let (encrypted_reader, encrypted_writer) = derive_encrypted_transport(
            ephemeral_private,
            request.content.x25519_public_key,
            reader,
            writer,
            false,
        );

        Ok(HandshakeResult {
            encrypted_reader,
            encrypted_writer,
            remote_p2p_key: request.content.from_p2p_key,
            remote_endpoint: SocketAddr::new(peer_ip, request.content.listening_port),
        })
    })
    .await?
}
fn validate_signature_and_timestamp(
    from_key: ed25519::PublicKey,
    content_hash: Sha256Digest,
    signature: ed25519::Signature,
    timestamp: u64,
) -> eyre::Result<()> {
    if !from_key.verify_sig(content_hash, signature) {
        return Err(eyre::eyre!("invalid signature"));
    }
    let their_time = UNIX_EPOCH + Duration::from_secs(timestamp);
    let now = SystemTime::now();
    let time_since_signature = now.duration_since(their_time).unwrap_or(Duration::ZERO);
    if time_since_signature > Duration::from_secs(30) {
        return Err(eyre::eyre!("invalid timestamp"));
    }
    Ok(())
}

fn generate_ephemeral_x25519_key() -> x25519::PrivateKey {
    let mut bytes = [0u8; 32];
    rng::fill_bytes(&mut bytes);
    x25519::PrivateKey::from_bytes(bytes)
}

fn derive_encrypted_transport(
    ephemeral_private: x25519::PrivateKey,
    remote_x25519_pub: x25519::PublicKey,
    reader: FramedReader,
    writer: FramedWriter,
    is_initiator: bool,
) -> (EncryptedReader, EncryptedWriter) {
    let shared_secret = ephemeral_private.diffie_hellman(remote_x25519_pub);
    let (send_cipher, recv_cipher) =
        TransportCipher::from_shared_secret(&shared_secret, is_initiator);
    (EncryptedReader::new(reader, recv_cipher), EncryptedWriter::new(writer, send_cipher))
}

pub struct HandshakeResult {
    pub encrypted_reader: EncryptedReader,
    pub encrypted_writer: EncryptedWriter,
    pub remote_p2p_key: ed25519::PublicKey,
    pub remote_endpoint: SocketAddr,
}

#[derive(BorshDeserialize, BorshSerialize, Clone, PartialEq, Debug)]
struct HandshakeRequestContent {
    from_p2p_key: ed25519::PublicKey,
    unix_timestamp: u64,
    to_p2p_key: ed25519::PublicKey,
    listening_port: u16,
    x25519_public_key: x25519::PublicKey,
}
impl HandshakeRequestContent {
    fn calculate_hash(&self) -> Sha256Digest {
        sha256::sha256_hash(&self.encode())
    }
}

#[derive(BorshDeserialize, BorshSerialize, Clone, Debug)]
struct HandshakeRequest {
    signature: ed25519::Signature,
    content: HandshakeRequestContent,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
struct HandshakeResponseContent {
    unix_timestamp: u64,
    listening_port: u16,
    request_hash: Sha256Digest,
    x25519_public_key: x25519::PublicKey,
}
impl HandshakeResponseContent {
    fn calculate_hash(&self) -> Sha256Digest {
        sha256::sha256_hash(&self.encode())
    }
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
struct HandshakeResponse {
    signature: ed25519::Signature,
    content: HandshakeResponseContent,
}

use crate::p2p::transport::{EncryptedReader, EncryptedWriter, FramedReader, FramedWriter};
use crate::p2p::transport_cipher::TransportCipher;
use crate::utils::rng;
use borsh::{BorshDeserialize, BorshSerialize};
use bytes::Bytes;
use vastrum_shared_types::borsh::BorshExt;
use vastrum_shared_types::crypto::{
    ed25519,
    sha256::{self, Sha256Digest},
    x25519,
};
use std::net::SocketAddr;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::net::TcpStream;
