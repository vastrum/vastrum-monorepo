#[wasm_bindgen(start)]
pub async fn main() -> Result<(), String> {
    console_error_panic_hook::set_once();
    start_iframe_rpc_listener()?;
    connect_to_rpc().await;

    Ok(())
}
/// Read __frontendData injected by the node into the frontend, contains rpc endpoints and genesis epoch state
pub fn read_frontend_data() -> vastrum_shared_types::frontend::frontend_data::FrontendData {
    let document = web_sys::window().unwrap().document().unwrap();
    let element = document.get_element_by_id("__frontendData").unwrap();
    let json = element.text_content().unwrap();
    serde_json::from_str(&json).unwrap()
}

#[wasm_bindgen]
pub async fn get_page(path: String, site_id: String) -> Result<JSPageResponse, String> {
    Ok(networking::rpc::get_page(path, site_id).await?)
}

pub mod crypto;
pub mod helios;
pub mod iframe_rpc;
pub mod networking;
pub mod utils;

use crate::{
    iframe_rpc::listener::start_iframe_rpc_listener,
    networking::rpc::{JSPageResponse, connect_to_rpc},
};
use wasm_bindgen::prelude::*;
