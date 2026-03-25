thread_local! {
    static TRANSPORT: RefCell<Option<RpcTransport>> = const { RefCell::new(None) };
}

const RECONNECT_DELAY_MS: u32 = 2_000;
const REQUEST_TIMEOUT_MS: u32 = 10_000;

pub async fn start_webrtc_connection(server_addr: SocketAddr, fingerprint: Fingerprint) {
    let _ = connect(server_addr, fingerprint).await;
    spawn_connection_loop(server_addr, fingerprint);
}

async fn connect(addr: SocketAddr, fp: Fingerprint) -> Result<()> {
    let pending = Rc::new(RefCell::new(HashMap::new()));
    let raw = WebRtcClient::connect(addr, fp).await?;
    let client = Rc::new(FramedClient::new(raw));
    TRANSPORT.with(|t| {
        *t.borrow_mut() = Some(RpcTransport { inner: client, pending, next_id: Cell::new(1) });
    });
    Ok(())
}

async fn recv_until_closed() {
    let Some((client, pending)) = TRANSPORT.with(|t| {
        let t = t.borrow();
        let transport = t.as_ref()?;
        Some((transport.inner.clone(), transport.pending.clone()))
    }) else {
        return;
    };
    while let Some(msg) = client.recv().await {
        let Ok(resp) = borsh::from_slice::<RpcResponse>(&msg) else {
            continue;
        };
        if let Some(tx) = pending.borrow_mut().remove(&resp.id) {
            let _ = tx.send(resp);
        }
    }
}

async fn reconnect(addr: SocketAddr, fp: Fingerprint) {
    TRANSPORT.with(|t| *t.borrow_mut() = None);
    loop {
        gloo_timers::future::TimeoutFuture::new(RECONNECT_DELAY_MS).await;
        if connect(addr, fp).await.is_ok() {
            return;
        }
    }
}

fn spawn_connection_loop(addr: SocketAddr, fp: Fingerprint) {
    wasm_bindgen_futures::spawn_local(async move {
        loop {
            recv_until_closed().await;
            reconnect(addr, fp).await;
        }
    });
}

pub async fn send_request(route: &str, body: &[u8]) -> Result<Vec<u8>> {
    let rx = TRANSPORT.with(|t| {
        let t = t.borrow();
        let transport = t.as_ref().ok_or(WasmErr::NotConnected)?;
        transport.send_request(route, body)
    })?;

    let timeout = gloo_timers::future::TimeoutFuture::new(REQUEST_TIMEOUT_MS);
    match futures::future::select(Box::pin(rx), Box::pin(timeout)).await {
        futures::future::Either::Left((Ok(resp), _)) => parse_response(resp),
        futures::future::Either::Left((Err(_), _)) => Err(WasmErr::ChannelClosed),
        futures::future::Either::Right(_) => Err(WasmErr::RequestTimeout),
    }
}

pub async fn send_fire_and_forget(route: &str, body: &[u8]) -> Result<()> {
    TRANSPORT.with(|t| {
        let t = t.borrow();
        let transport = t.as_ref().ok_or(WasmErr::NotConnected)?;
        transport.send_fire_and_forget(route, body)
    })
}

fn parse_response(resp: RpcResponse) -> Result<Vec<u8>> {
    match resp.body {
        RpcBody::Success(body) => Ok(body),
        RpcBody::Error(msg) => Err(WasmErr::RpcError(msg)),
    }
}

use super::transport::RpcTransport;
use crate::utils::error::{Result, WasmErr};
use vastrum_shared_types::types::rpc::types::{RpcBody, RpcResponse};
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::rc::Rc;
use webrtc_direct_client::{Fingerprint, FramedClient, WebRtcClient};
