#[cfg(all(feature = "eth-rpc-webrtc", feature = "eth-rpc-http"))]
compile_error!("features `eth-rpc-webrtc` and `eth-rpc-http` are mutually exclusive");

thread_local! {
    static STATE: RefCell<WorkerState> = RefCell::new(WorkerState {
        status: WorkerStatus::Idle,
        pending_requests: HashMap::new(),
        next_request_id: 1,
    });
}

pub async fn send_eth_rpc_to_worker(req: EthRPCRequest) -> Value {
    let Some(worker) = await_worker_ready().await else {
        return rpc_error("worker init failed");
    };

    let id = next_request_id();
    let (tx, rx) = oneshot::channel();

    STATE.with(|s| {
        s.borrow_mut().pending_requests.insert(id, tx);
    });

    let json = serde_json::to_string(&req).unwrap();

    let msg = serde_wasm_bindgen::to_value(&WorkerRequest { id, request: json }).unwrap();

    worker.post_message(&msg).unwrap();

    let response = rx.await;
    match response {
        Ok(value) => value,
        Err(_) => rpc_error("worker crashed"),
    }
}
async fn await_worker_ready() -> Option<Worker> {
    let is_idle = STATE.with(|s| matches!(s.borrow().status, WorkerStatus::Idle));
    if is_idle {
        init_worker();
    }
    loop {
        let result = STATE.with(|s| match &s.borrow().status {
            WorkerStatus::Ready(w) => Some(w.clone()),
            _ => None,
        });
        if let Some(worker) = result {
            return Some(worker);
        }
        let worker_crashed = STATE.with(|s| matches!(s.borrow().status, WorkerStatus::Idle));
        if worker_crashed {
            return None;
        }
        gloo_timers::future::sleep(std::time::Duration::from_millis(10)).await;
    }
}
fn init_worker() {
    let worker = spawn_helios_worker();

    let onmessage = Closure::<dyn Fn(MessageEvent)>::new(handle_worker_message);
    worker.set_onmessage(Some(onmessage.as_ref().unchecked_ref()));
    onmessage.forget();

    let onerror = Closure::<dyn Fn(ErrorEvent)>::new(handle_worker_error);
    worker.set_onerror(Some(onerror.as_ref().unchecked_ref()));
    onerror.forget();

    STATE.with(|s| s.borrow_mut().status = WorkerStatus::Starting(worker));
}
fn spawn_helios_worker() -> Worker {
    let js = build_worker_js();
    let props = BlobPropertyBag::new();
    props.set_type("application/javascript");
    let blob = Blob::new_with_str_sequence_and_options(
        &js_sys::Array::of1(&JsValue::from_str(&js)),
        &props,
    )
    .unwrap();
    let url = Url::create_object_url_with_blob(&blob).unwrap();
    let opts = web_sys::WorkerOptions::new();
    opts.set_type(web_sys::WorkerType::Module);
    let worker = Worker::new_with_options(&url, &opts).unwrap();
    Url::revoke_object_url(&url).unwrap();
    worker
}

fn build_worker_js() -> String {
    let base_url = get_rpc_endpoint("");
    let base_url = base_url.trim_end_matches('/');
    let checkpoint = crate::read_frontend_data().helios_checkpoint;
    let origin = web_sys::window().unwrap().location().origin().unwrap();

    #[cfg(feature = "eth-rpc-webrtc")]
    const WORKER_JS: &str = include_str!("helios_worker_webrtc.js");
    #[cfg(feature = "eth-rpc-http")]
    const WORKER_JS: &str = include_str!("helios_worker_http.js");

    WORKER_JS
        .replace("__ORIGIN__", &origin)
        .replace("__EXECUTION_RPC__", &format!("{base_url}/ethexecutionrpc"))
        .replace("__CONSENSUS_RPC__", &format!("{base_url}/ethconsensusrpc"))
        .replace("__CHECKPOINT__", &checkpoint)
        .replace("__NETWORK__", "mainnet")
}

fn handle_worker_message(e: MessageEvent) {
    let msg = serde_wasm_bindgen::from_value::<WorkerMessage>(e.data()).unwrap();

    match msg {
        WorkerMessage::Ready => {
            STATE.with(|s| s.borrow_mut().set_ready());
        }
        WorkerMessage::Error { error } => {
            STATE.with(|s| s.borrow_mut().reset_worker(&format!("worker init error: {error}")));
        }
        WorkerMessage::Response { id, data } => {
            let value: Value = serde_json::from_str(&data).unwrap();
            STATE.with(|s| {
                if let Some(tx) = s.borrow_mut().pending_requests.remove(&id) {
                    let _ = tx.send(value);
                }
            });
        }
        WorkerMessage::FetchIntercept { id, url, method, body } => {
            let worker = STATE.with(|s| match &s.borrow().status {
                WorkerStatus::Ready(w) | WorkerStatus::Starting(w) => Some(w.clone()),
                _ => None,
            });
            if let Some(worker) = worker {
                wasm_bindgen_futures::spawn_local(async move {
                    let resp = rpc::eth_proxy(url, method, body.unwrap_or_default())
                        .await
                        .unwrap_or(EthProxyResponse {
                            status: 502,
                            body: br#"{"error":"proxy error"}"#.to_vec(),
                            content_type: "application/json".into(),
                        });
                    let msg = FetchResponseMsg {
                        msg_type: "FetchResponse".into(),
                        id: id as u32,
                        status: resp.status,
                        body: resp.body,
                        content_type: resp.content_type,
                    };
                    worker.post_message(&serde_wasm_bindgen::to_value(&msg).unwrap()).unwrap();
                });
            }
        }
    }
}
fn next_request_id() -> u64 {
    STATE.with(|s| {
        let mut state = s.borrow_mut();
        let id = state.next_request_id;
        state.next_request_id += 1;
        id
    })
}
fn handle_worker_error(e: ErrorEvent) {
    let msg = e.message();
    STATE.with(|s| s.borrow_mut().reset_worker(&format!("worker error: {msg}")));
}

enum WorkerStatus {
    Idle,
    Starting(Worker),
    Ready(Worker),
}

struct WorkerState {
    status: WorkerStatus,
    pending_requests: HashMap<u64, oneshot::Sender<Value>>,
    next_request_id: u64,
}

impl WorkerState {
    fn set_ready(&mut self) {
        let old = std::mem::replace(&mut self.status, WorkerStatus::Idle);
        if let WorkerStatus::Starting(w) = old {
            self.status = WorkerStatus::Ready(w);
        }
    }

    fn reset_worker(&mut self, error_msg: &str) {
        self.status = WorkerStatus::Idle;
        let pending_requests = std::mem::take(&mut self.pending_requests);
        for (_, tx) in pending_requests {
            let _ = tx.send(rpc_error(error_msg));
        }
    }
}

#[derive(Deserialize)]
#[serde(tag = "type")]
enum WorkerMessage {
    Ready,
    Error { error: String },
    Response { id: u64, data: String },
    FetchIntercept { id: u64, url: String, method: String, body: Option<Vec<u8>> },
}

#[derive(Serialize)]
struct FetchResponseMsg {
    #[serde(rename = "type")]
    msg_type: String,
    id: u32,
    status: u16,
    body: Vec<u8>,
    content_type: String,
}

#[derive(Serialize)]
struct WorkerRequest {
    id: u64,
    request: String,
}

fn rpc_error(msg: &str) -> Value {
    serde_json::json!({"error": {"code": -32603, "message": msg}})
}

use crate::networking::rpc::{self, get_rpc_endpoint};
use futures::channel::oneshot;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use vastrum_shared_types::iframerpc::types::EthRPCRequest;
use vastrum_shared_types::types::rpc::types::EthProxyResponse;
use std::cell::RefCell;
use std::collections::HashMap;
use wasm_bindgen::prelude::*;
use web_sys::{Blob, BlobPropertyBag, ErrorEvent, MessageEvent, Url, Worker, js_sys};
