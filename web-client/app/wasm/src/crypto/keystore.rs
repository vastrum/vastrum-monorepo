pub fn get_account_private_key() -> Result<ed25519::PrivateKey> {
    let window = window().unwrap();
    let storage = window.local_storage()?.ok_or(WasmErr::BrowserApi("local storage"))?;

    if let Ok(Some(value)) = storage.get_item("account_private_key") {
        let account_private_key: ed25519::PrivateKey = serde_json::from_str(&value)?;
        return Ok(account_private_key);
    } else {
        let account_private_key = generate_private_key();
        let serialized = serde_json::to_string(&account_private_key).unwrap();
        let _ = storage.set_item("account_private_key", &serialized);
        return Ok(account_private_key);
    }
}

pub fn get_ed25519_public_key() -> Result<ed25519::PublicKey> {
    return Ok(get_account_private_key()?.public_key());
}

pub fn get_site_private_key() -> Result<ed25519::PrivateKey> {
    let site_id = get_current_site_id()?;
    let master_key = get_account_private_key()?;
    return Ok(derive_site_key(&master_key, site_id));
}

pub fn generate_private_key() -> ed25519::PrivateKey {
    return ed25519::PrivateKey::from_rng();
}

use crate::utils::error::{Result, WasmErr};
use crate::utils::site_id::get_current_site_id;
use vastrum_shared_types::crypto::ed25519;
use vastrum_shared_types::crypto::site_key::derive_site_key;
use web_sys::window;
