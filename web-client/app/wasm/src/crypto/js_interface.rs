#[wasm_bindgen]
pub fn set_private_key(private_key: String) -> Result<(), String> {
    Ok(set_private_key_inner(private_key)?)
}

fn set_private_key_inner(private_key: String) -> Result<(), WasmErr> {
    let window = window().unwrap();
    let storage = window.local_storage()?.ok_or(WasmErr::BrowserApi("local storage"))?;

    let key = PrivateKey::try_from_string(private_key);
    if let Some(key) = key {
        let serialized = serde_json::to_string(&key).unwrap();
        let _ = storage.set_item("account_private_key", &serialized);
    }
    return Ok(());
}

#[wasm_bindgen]
pub fn get_private_key() -> Result<String, String> {
    let key = keystore::get_account_private_key()?;
    return Ok(key.to_string());
}

#[wasm_bindgen]
pub fn get_ed25519_public_key() -> Result<String, String> {
    let key = keystore::get_ed25519_public_key()?;
    return Ok(key.to_string());
}

use crate::crypto::keystore;
use crate::utils::error::WasmErr;
use vastrum_shared_types::crypto::ed25519::PrivateKey;
use wasm_bindgen::prelude::wasm_bindgen;
use web_sys::window;
