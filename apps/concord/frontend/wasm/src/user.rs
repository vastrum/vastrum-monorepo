use crate::SentTxBehavior;
use crate::types::JSUserProfile;
use vastrum_frontend_lib::get_pub_key;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub async fn get_my_pubkey() -> String {
    let public_key = get_pub_key().await;
    let pubkey_hex = hex::encode(public_key.to_bytes());
    return pubkey_hex;
}

#[wasm_bindgen]
pub async fn set_display_name(name: String) -> String {
    let client = crate::new_client();
    let sent_tx = client.set_display_name(name).await;
    let tx_hash = sent_tx.tx_hash().to_string();
    return tx_hash;
}

#[wasm_bindgen]
pub async fn get_user_profile(hex_pubkey: String) -> JSUserProfile {
    let client = crate::new_client();
    let state = client.state().await;
    let public_key = crate::string_to_pubkey(&hex_pubkey);
    let profile = match state.user_profiles.get(&public_key).await {
        Some(p) => p,
        None => {
            return JSUserProfile {
                display_name: String::new(),
                server_ids: vec![],
                dm_keys: vec![],
            };
        }
    };
    let display_name = profile.display_name.clone().unwrap_or_default();
    let partner_count = profile.dm_activity.length().await;
    let dm_entries = profile.dm_activity.get_descending_entries(partner_count as usize, 0).await;
    let mut dm_keys = Vec::new();
    for dk in &dm_entries {
        let other = crate::dm_other(dk, public_key);
        dm_keys.push(hex::encode(other.to_bytes()));
    }
    JSUserProfile {
        display_name,
        server_ids: profile.encrypted_server_keys.iter().map(|(id, _)| *id).collect(),
        dm_keys,
    }
}
