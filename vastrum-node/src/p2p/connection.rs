#[derive(Clone)]
pub struct PeerSender {
    write_tx: mpsc::Sender<Vec<u8>>,
    pending: Arc<Mutex<HashMap<u64, oneshot::Sender<Response>>>>,
    next_request_id: Arc<AtomicU64>,
}

impl PeerSender {
    pub fn send_statement(&self, payload: Vec<u8>) -> Result<(), TrySendError<Vec<u8>>> {
        let msg = Message::Statement { payload };
        self.write_tx.try_send(msg.encode())?;
        return Ok(());
    }

    pub async fn send_request(
        &self,
        payload: Vec<u8>,
        timeout_secs: u64,
    ) -> eyre::Result<Response> {
        let id = self.next_request_id.fetch_add(1, Ordering::Relaxed);
        let (tx, rx) = oneshot::channel();
        self.pending.lock().insert(id, tx);
        let data = Message::Request { id, payload }.encode();
        self.write_tx.try_send(data)?;
        let response = tokio::time::timeout(Duration::from_secs(timeout_secs), rx).await;
        if let Ok(Ok(response)) = response {
            return Ok(response);
        } else {
            //clean up pending request
            self.pending.lock().remove(&id);
            return Err(eyre::eyre!("request timed out or could not be handled"));
        }
    }
}

struct RateLimiter {
    tokens_left: f64,
    max_stored_tokens: f64,
    max_tokens_per_second: f64,
    last_refill: Instant,
}

impl RateLimiter {
    fn new(max_tokens_per_second: f64) -> Self {
        let max_stored_tokens = max_tokens_per_second;
        RateLimiter {
            tokens_left: max_stored_tokens,
            max_stored_tokens,
            max_tokens_per_second,
            last_refill: Instant::now(),
        }
    }

    fn try_consume_tokens(&mut self) -> bool {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill).as_secs_f64();
        let regen_tokens = elapsed * self.max_tokens_per_second;
        self.tokens_left += regen_tokens;
        //max requests stored
        self.tokens_left = f64::min(self.tokens_left, self.max_stored_tokens);

        self.last_refill = now;
        if self.tokens_left >= 1.0 {
            self.tokens_left -= 1.0;
            return true;
        } else {
            return false;
        }
    }
}

pub fn spawn_connection_actor(
    encrypted_reader: EncryptedReader,
    encrypted_writer: EncryptedWriter,
    remote_p2p_key: ed25519::PublicKey,
    inbound_tx: mpsc::UnboundedSender<InboundMessage>,
    release_tx: mpsc::UnboundedSender<ed25519::PublicKey>,
    max_tokens_per_second: f64,
) -> PeerSender {
    let (write_tx, write_rx) = mpsc::channel::<Vec<u8>>(32);
    let (pong_nonce_tx, pong_nonce_rx) = watch::channel(0u64);
    let pending: Arc<Mutex<HashMap<u64, oneshot::Sender<Response>>>> =
        Arc::new(Mutex::new(HashMap::new()));

    let reader_write_tx = write_tx.clone();
    let heartbeat_write_tx = write_tx.clone();
    let reader_pending = pending.clone();
    let rate_limiter = RateLimiter::new(max_tokens_per_second);

    tokio::spawn(async move {
        tokio::select! { biased;
            _ = writer_task(encrypted_writer, write_rx) => {}
            _ = reader_task(
                encrypted_reader,
                reader_pending,
                reader_write_tx,
                inbound_tx,
                pong_nonce_tx,
                rate_limiter,
            ) => {}
            _ = heartbeat_task(heartbeat_write_tx, pong_nonce_rx) => {}
        }
        let _ = release_tx.send(remote_p2p_key);
    });

    PeerSender { write_tx, pending, next_request_id: Arc::new(AtomicU64::new(0)) }
}

async fn writer_task(mut writer: EncryptedWriter, mut write_rx: mpsc::Receiver<Vec<u8>>) {
    while let Some(data) = write_rx.recv().await {
        if writer.write_frame(&data).await.is_err() {
            break;
        }
    }
}

async fn reader_task(
    mut reader: EncryptedReader,
    pending: Arc<Mutex<HashMap<u64, oneshot::Sender<Response>>>>,
    write_tx: mpsc::Sender<Vec<u8>>,
    inbound_tx: mpsc::UnboundedSender<InboundMessage>,
    pong_nonce_tx: watch::Sender<u64>,
    mut rate_limiter: RateLimiter,
) {
    loop {
        let Ok(frame) = reader.read_frame().await else { break };

        if !rate_limiter.try_consume_tokens() {
            continue;
        }

        let Ok(message) = Message::decode(&Bytes::from(frame)) else {
            break;
        };

        match message {
            Message::Request { id, payload } => {
                let handle = ResponseHandle::new(write_tx.clone(), id);
                let _ = inbound_tx.send(InboundMessage { payload, respond: Some(handle) });
            }
            Message::Response { id, payload } => {
                if let Some(sender) = pending.lock().remove(&id) {
                    let _ = sender.send(Response { payload });
                }
            }
            Message::Statement { payload } => {
                let _ = inbound_tx.send(InboundMessage { payload, respond: None });
            }
            Message::Ping { nonce } => {
                let pong = Message::Pong { nonce };
                let _ = write_tx.try_send(pong.encode());
            }
            Message::Pong { nonce } => {
                let _ = pong_nonce_tx.send(nonce);
            }
        }
    }
}

async fn heartbeat_task(write_tx: mpsc::Sender<Vec<u8>>, pong_nonce_rx: watch::Receiver<u64>) {
    let timeout_limit = Duration::from_secs(20);
    let mut last_confirmed = Instant::now();
    let mut nonce: u64 = 1;
    loop {
        sleep(Duration::from_secs(3)).await;
        if *pong_nonce_rx.borrow() >= nonce - 1 {
            last_confirmed = Instant::now();
        }
        if last_confirmed.elapsed() > timeout_limit {
            break;
        }
        let ping = Message::Ping { nonce };
        let _ = write_tx.try_send(ping.encode());
        nonce += 1;
    }
}

pub struct ResponseHandle {
    write_tx: mpsc::Sender<Vec<u8>>,
    request_id: u64,
}
impl ResponseHandle {
    pub fn new(write_tx: mpsc::Sender<Vec<u8>>, request_id: u64) -> Self {
        ResponseHandle { write_tx, request_id }
    }
    pub fn respond(self, payload: Vec<u8>) {
        let data = Message::Response { id: self.request_id, payload }.encode();
        let _ = self.write_tx.try_send(data);
    }
}

use crate::p2p::transport::{EncryptedReader, EncryptedWriter};
use crate::p2p::types::messages::{InboundMessage, Message, Response};
use bytes::Bytes;
use parking_lot::Mutex;
use vastrum_shared_types::borsh::BorshExt;
use vastrum_shared_types::crypto::ed25519;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;
use tokio::sync::mpsc::error::TrySendError;
use tokio::sync::{mpsc, oneshot, watch};
use tokio::time::{Instant, sleep};
