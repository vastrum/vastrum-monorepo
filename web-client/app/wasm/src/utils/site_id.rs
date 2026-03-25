use crate::utils::error::{Result, WasmErr};
use vastrum_shared_types::crypto::sha256::Sha256Digest;
use std::cell::RefCell;

thread_local! {
    static SITE_ID: RefCell<Option<Sha256Digest>> = const { RefCell::new(None) };
}

pub fn get_current_site_id() -> Result<Sha256Digest> {
    let site_id = SITE_ID.with(|s| *s.borrow());
    let Some(site_id) = site_id else {
        return Err(WasmErr::BrowserApi("site id not set"));
    };
    return Ok(site_id);
}

pub fn set_current_site_id(site_id: Sha256Digest) {
    SITE_ID.with(|s| *s.borrow_mut() = Some(site_id));
}
