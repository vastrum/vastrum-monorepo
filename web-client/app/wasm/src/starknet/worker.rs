thread_local! {
    static STATE: RefCell<WorkerState> = RefCell::new(WorkerState {
        status: WorkerStatus::Idle,
        pending_requests: HashMap::new(),
        next_request_id: 1,
    });
}

pub async fn send_starknet_rpc_to_worker(req: StarknetRPCRequest) -> Value {
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
    let worker = spawn_beerus_worker();

    let onmessage = Closure::<dyn Fn(MessageEvent)>::new(handle_worker_message);
    worker.set_onmessage(Some(onmessage.as_ref().unchecked_ref()));
    onmessage.forget();

    let onerror = Closure::<dyn Fn(ErrorEvent)>::new(handle_worker_error);
    worker.set_onerror(Some(onerror.as_ref().unchecked_ref()));
    onerror.forget();

    STATE.with(|s| s.borrow_mut().status = WorkerStatus::Starting(worker));
}

fn spawn_beerus_worker() -> Worker {
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
    let origin = web_sys::window().unwrap().location().origin().unwrap();

    const WORKER_JS: &str = include_str!("beerus_worker.js");

    WORKER_JS.replace("__ORIGIN__", &origin)
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
}

#[derive(Serialize)]
struct WorkerRequest {
    id: u64,
    request: String,
}

fn rpc_error(msg: &str) -> Value {
    serde_json::json!({"error": {"code": -32603, "message": msg}})
}

use futures::channel::oneshot;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use vastrum_shared_types::iframerpc::types::StarknetRPCRequest;
use std::cell::RefCell;
use std::collections::HashMap;
use wasm_bindgen::prelude::*;
use web_sys::{Blob, BlobPropertyBag, ErrorEvent, MessageEvent, Url, Worker, js_sys};
