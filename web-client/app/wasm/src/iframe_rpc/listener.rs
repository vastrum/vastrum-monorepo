pub fn start_iframe_rpc_listener() -> Result<()> {
    start_iframe_rpc_message_handler()?;
    start_navigation_sync()?;
    return Ok(());
}

fn start_iframe_rpc_message_handler() -> Result<()> {
    let window = web_sys::window().unwrap();
    let handler = Closure::wrap(Box::new(move |event: MessageEvent| {
        spawn_local(async move {
            if let Some(payload) = event.data().as_string() {
                if let Ok(data) = serde_json::from_str::<RpcRequest>(&payload) {
                    handle_request(data).await;
                }
            }
        });
    }) as Box<dyn FnMut(MessageEvent)>);
    window.add_event_listener_with_callback("message", handler.as_ref().unchecked_ref())?;
    handler.forget();
    return Ok(());
}

//forward backward browser page navigation api handler
fn start_navigation_sync() -> Result<()> {
    let window = web_sys::window().unwrap();
    let closure = Closure::wrap(Box::new(|_: web_sys::PopStateEvent| {
        let window = web_sys::window().unwrap();
        if let Ok(path) = window.location().pathname() {
            send_page_navigation_event(path);
        }
    }) as Box<dyn FnMut(_)>);
    window.add_event_listener_with_callback("popstate", closure.as_ref().unchecked_ref())?;
    closure.forget();
    return Ok(());
}

fn get_iframe() -> Result<HtmlIFrameElement> {
    let document = web_sys::window().unwrap().document().unwrap();
    let el = document.query_selector("iframe")?.ok_or(WasmErr::BrowserApi("iframe element"))?;
    Ok(el.unchecked_into())
}

async fn handle_request(request: RpcRequest) {
    spawn_local(async move {
        let response_payload = match handle_request_methods(&request).await {
            Ok(payload) => payload,
            Err(e) => {
                format!(r#"{{"error":"{}"}}"#, e)
            }
        };

        let response = RpcResponse {
            request_id: request.request_id,
            method: RpcMethodHostToIFrame::Response,
            params: response_payload,
        };

        let response_serialized = serde_json::to_string(&response).unwrap();
        let Ok(iframe) = get_iframe() else { return };
        let Some(window) = iframe.content_window() else { return };
        let _ = window.post_message(&JsValue::from_str(&response_serialized), "*");
    });
}

async fn handle_request_methods(request: &RpcRequest) -> Result<String> {
    use handlers::*;

    return match request.method {
        RpcMethod::GetKeyValue => {
            let req = serde_json::from_str(&request.params)?;
            let res = handle_get_key_value(req).await?;
            Ok(serde_json::to_string(&res).unwrap())
        }
        RpcMethod::MakeCall => {
            let params = serde_json::from_str(&request.params)?;
            let res = make_call(params).await?;
            Ok(serde_json::to_string(&res).unwrap())
        }
        RpcMethod::MakeAuthenticatedCall => {
            let params = serde_json::from_str(&request.params)?;
            let res = make_authenticated_call(params).await?;
            Ok(serde_json::to_string(&res).unwrap())
        }
        RpcMethod::GetPrivateSalt => {
            let params = serde_json::from_str(&request.params)?;
            let res = get_private_salt_for_site_id(params).await?;
            Ok(serde_json::to_string(&res).unwrap())
        }
        RpcMethod::GetSitePubKey => {
            let params = serde_json::from_str(&request.params)?;
            let res = get_site_pub_key(params).await?;
            Ok(serde_json::to_string(&res).unwrap())
        }
        RpcMethod::GetTxHashIsIncluded => {
            let params = serde_json::from_str(&request.params)?;
            let res = get_tx_hash_is_included(params).await?;
            Ok(serde_json::to_string(&res).unwrap())
        }
        RpcMethod::GetCurrentPath => {
            let params = serde_json::from_str(&request.params)?;
            let res = get_current_path(params);
            Ok(serde_json::to_string(&res).unwrap())
        }
        RpcMethod::UpdateCurrentPath => {
            let params = serde_json::from_str(&request.params)?;
            let res = update_current_path(params).await?;
            Ok(serde_json::to_string(&res).unwrap())
        }
        RpcMethod::EthRpcRequest => {
            let params = serde_json::from_str(&request.params)?;
            let res = handle_eth_rpc_request(params).await;
            Ok(serde_json::to_string(&res).unwrap())
        }
        RpcMethod::GetKeyValueBySiteId => {
            let params: GetKeyValueBySiteIdRequest = serde_json::from_str(&request.params)?;
            let rpc_res = get_key_value(params.site_id, params.key).await?;
            let iframe_res = GetKeyValueResponse { value: rpc_res.value };
            Ok(serde_json::to_string(&iframe_res).unwrap())
        }
        RpcMethod::GetLatestBlockHeight => {
            let res = handlers::handle_get_latest_block_height().await?;
            Ok(serde_json::to_string(&res).unwrap())
        }
        RpcMethod::GetSitePrivateKey => {
            let params = serde_json::from_str(&request.params)?;
            let res = handlers::get_site_private_key(params).await?;
            Ok(serde_json::to_string(&res).unwrap())
        }
        RpcMethod::OpenExternalUrl => {
            let params: OpenExternalUrlRequest = serde_json::from_str(&request.params)?;
            if !params.url.starts_with("https://") {
                return Err(WasmErr::InvalidUrl);
            }
            let window = web_sys::window().unwrap();
            let init = web_sys::CustomEventInit::new();
            init.set_detail(&JsValue::from_str(&params.url));
            //event handled by web-client/src/external_link_modal.tsx
            let event =
                web_sys::CustomEvent::new_with_event_init_dict("vastrum_open_external_url", &init)?;
            window.dispatch_event(&event)?;
            Ok(serde_json::to_string(&OpenExternalUrlResponse {}).unwrap())
        }
    };
}

fn send_page_navigation_event(path: String) {
    let Ok(iframe) = get_iframe() else { return };
    let Some(window) = iframe.content_window() else { return };

    let json = serde_json::to_string(&PageNavigationEventMessage { path }).unwrap();
    let response = RpcResponse {
        request_id: 0,
        method: RpcMethodHostToIFrame::PageNavigationEvent,
        params: json,
    };
    let serialized = serde_json::to_string(&response).unwrap();
    let _ = window.post_message(&JsValue::from_str(&serialized), "*");
}

use super::handlers;
use crate::networking::rpc::get_key_value;
use crate::utils::error::{Result, WasmErr};
use vastrum_shared_types::iframerpc::types::*;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::Closure;
use wasm_bindgen_futures::spawn_local;
use web_sys::{HtmlIFrameElement, MessageEvent};
