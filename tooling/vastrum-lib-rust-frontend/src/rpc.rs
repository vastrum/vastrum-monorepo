pub type PendingRequests = Rc<Mutex<HashMap<u64, oneshot::Sender<String>>>>;

thread_local! {
    static RPC_STATE: RefCell<Option<PendingRequests>> = const { RefCell::new(None) };
}

fn get_or_init_rpc() -> PendingRequests {
    RPC_STATE.with(|cell| {
        let mut rpc_state = cell.borrow_mut();
        let listener_has_not_yet_started = rpc_state.is_none();
        if listener_has_not_yet_started {
            console_error_panic_hook::set_once();

            let pending_requests: PendingRequests = Rc::new(Mutex::new(HashMap::new()));

            let pending_clone = pending_requests.clone();
            let handler = Closure::wrap(Box::new(move |event: MessageEvent| {
                let pending = pending_clone.clone();
                spawn_local(async move {
                    if let Some(payload) = event.data().as_string() {
                        if let Ok(data) = serde_json::from_str::<RpcResponse>(&payload) {
                            handle_response(data, pending).await;
                        }
                    }
                });
            }) as Box<dyn FnMut(MessageEvent)>);

            let window = web_sys::window().unwrap();
            window
                .add_event_listener_with_callback("message", handler.as_ref().unchecked_ref())
                .unwrap();
            handler.forget();

            *rpc_state = Some(pending_requests);
        }
        rpc_state.as_ref().unwrap().clone()
    })
}

async fn handle_response(data: RpcResponse, pending_requests: PendingRequests) {
    match data.method {
        RpcMethodHostToIFrame::Response => {
            if let Some(sender) = pending_requests.lock().await.remove(&data.request_id) {
                let _ = sender.send(data.params);
            }
        }
        RpcMethodHostToIFrame::PageNavigationEvent => navigate_to(&data.params),
    }
}

pub async fn send_request<RequestType, ReturnType>(
    params: RequestType,
    method: RpcMethod,
) -> Result<ReturnType, ()>
where
    RequestType: Serialize,
    ReturnType: for<'a> Deserialize<'a>,
{
    let pending_requests = get_or_init_rpc();
    let window = web_sys::window().unwrap();

    let params = serde_json::to_string(&params).unwrap();
    let request_id: u64 = rand::random();

    let payload = RpcRequest { request_id, method, params };
    let serialized_payload = serde_json::to_string(&payload).unwrap();
    let serialized_payload = &JsValue::from_str(&serialized_payload);

    let (tx, rx) = oneshot::channel();
    {
        pending_requests.lock().await.insert(request_id, tx);
    }

    let parent = window.parent().unwrap().unwrap();
    let _ = parent.post_message(serialized_payload, "*");

    let timeout = TimeoutFuture::new(60_000);
    let result = futures::future::select(rx, timeout).await;

    match result {
        futures::future::Either::Left((Ok(response), _)) => {
            match serde_json::from_str::<ReturnType>(&response) {
                Ok(parsed) => Ok(parsed),
                Err(e) => {
                    web_sys::console::error_1(&JsValue::from_str(&format!(
                        "[RPC deserialize error] {e} | raw response: {response}"
                    )));
                    Err(())
                }
            }
        }
        futures::future::Either::Left((Err(_), _)) => {
            web_sys::console::error_1(&JsValue::from_str("Response channel closed"));
            pending_requests.lock().await.remove(&request_id);
            Err(())
        }
        futures::future::Either::Right((_, _)) => {
            web_sys::console::error_1(&JsValue::from_str("Request timed out"));
            pending_requests.lock().await.remove(&request_id);
            Err(())
        }
    }
}

extern crate console_error_panic_hook;
use crate::handlers::navigate_to;
use futures::channel::oneshot;
use futures::lock::Mutex;
use gloo_timers::future::TimeoutFuture;
use serde::{Deserialize, Serialize};
use vastrum_shared_types::iframerpc::types::{RpcMethod, RpcMethodHostToIFrame, RpcRequest, RpcResponse};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use web_sys::MessageEvent;
