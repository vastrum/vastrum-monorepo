use crate::SentTxBehavior;
use crate::encryption::{encrypt_server_key, load_server_key, try_decrypt_content};
use crate::types::{JSChannel, JSMessage, JSServerDetail, JSServerSummary};
use vastrum_shared_types::borsh::BorshExt;
use vastrum_shared_types::crypto::cha_cha20_poly1305::ChaCha20Poly1305;
use vastrum_frontend_lib::get_pub_key;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub async fn create_server(name: String) -> String {
    let server_key = ChaCha20Poly1305::from_rng();
    let key_bytes = *server_key.to_bytes();
    let encrypted = encrypt_server_key(&key_bytes).await;
    let client = crate::new_client();
    let sent_tx = client.create_server(name, encrypted).await;
    let tx_hash = sent_tx.tx_hash().to_string();
    return tx_hash;
}

#[wasm_bindgen]
pub async fn get_my_servers() -> Vec<JSServerSummary> {
    let client = crate::new_client();
    let public_key = get_pub_key().await;
    let state = client.state().await;
    let profile = match state.user_profiles.get(&public_key).await {
        Some(p) => p,
        None => return vec![],
    };
    let mut servers = Vec::new();
    for (server_id, _) in &profile.encrypted_server_keys {
        if let Some(server) = state.servers.get(*server_id).await {
            servers.push(JSServerSummary {
                id: server.id,
                name: server.name,
                owner: server.owner.to_string(),
                member_count: server.members.length().await,
            });
        }
    }
    return servers;
}

#[wasm_bindgen]
pub async fn get_server(server_id: u64) -> Option<JSServerDetail> {
    let client = crate::new_client();
    let state = client.state().await;
    let server = state.servers.get(server_id).await?;
    let member_entries = server.members.get_ascending_entries(500, 0).await;
    let mut members = Vec::new();
    for (pubkey, _) in &member_entries {
        let name = state
            .user_profiles
            .get(pubkey)
            .await
            .and_then(|p| p.display_name)
            .unwrap_or_default();
        members.push(crate::types::JSMember { pubkey: pubkey.to_string(), display_name: name });
    }
    let mut channels = Vec::new();
    for channel in &server.channels {
        channels.push(JSChannel {
            id: channel.id,
            name: channel.name.clone(),
            message_count: channel.messages.length().await,
            next_message_id: channel.messages.length().await,
        });
    }
    let detail = JSServerDetail {
        id: server.id,
        name: server.name,
        owner: server.owner.to_string(),
        members,
        channels,
    };
    return Some(detail);
}

#[wasm_bindgen]
pub async fn join_server(server_id: u64, server_key_hex: String) -> String {
    let client = crate::new_client();
    if let Ok(key_bytes) = hex::decode(&server_key_hex) {
        if key_bytes.len() == 32 {
            let mut key = [0u8; 32];
            key.copy_from_slice(&key_bytes);
            let encrypted = encrypt_server_key(&key).await;
            let sent_tx = client.join_server(server_id, encrypted).await;
            return sent_tx.tx_hash().to_string();
        }
    }
    return String::new();
}

#[wasm_bindgen]
pub async fn leave_server(server_id: u64) -> String {
    let client = crate::new_client();
    let sent_tx = client.leave_server(server_id).await;
    let tx_hash = sent_tx.tx_hash().to_string();
    return tx_hash;
}

#[wasm_bindgen]
pub async fn create_channel(server_id: u64, name: String) -> String {
    let client = crate::new_client();
    let sent_tx = client.create_channel(server_id, name).await;
    let tx_hash = sent_tx.tx_hash().to_string();
    return tx_hash;
}

#[wasm_bindgen]
pub async fn delete_channel(server_id: u64, channel_id: u64) -> String {
    let client = crate::new_client();
    let sent_tx = client.delete_channel(server_id, channel_id).await;
    let tx_hash = sent_tx.tx_hash().to_string();
    return tx_hash;
}

#[wasm_bindgen]
pub async fn get_channels(server_id: u64) -> Vec<JSChannel> {
    let client = crate::new_client();
    let state = client.state().await;
    let server = match state.servers.get(server_id).await {
        Some(found_server) => found_server,
        None => return vec![],
    };
    let mut channels = Vec::new();
    for channel in &server.channels {
        channels.push(JSChannel {
            id: channel.id,
            name: channel.name.clone(),
            message_count: channel.messages.length().await,
            next_message_id: channel.messages.length().await,
        });
    }
    return channels;
}

#[wasm_bindgen]
pub async fn get_messages(server_id: u64, channel_id: u64, count: u64) -> Vec<JSMessage> {
    let client = crate::new_client();
    let state = client.state().await;
    let server = match state.servers.get(server_id).await {
        Some(found_server) => found_server,
        None => return vec![],
    };
    let channel = match server.channels.iter().find(|c| c.id == channel_id) {
        Some(c) => c,
        None => return vec![],
    };
    let mut messages = channel.messages.get_descending_entries(count as usize, 0).await;
    messages.reverse();

    let server_key = load_server_key(server_id).await;
    let cipher = server_key.map(ChaCha20Poly1305::from);

    let mut js_messages = Vec::new();
    for msg in &messages {
        let content = match &cipher {
            Some(decryptor) => try_decrypt_content(decryptor, &msg.content),
            None => msg.content.clone(),
        };
        js_messages.push(JSMessage {
            id: msg.id,
            content,
            author: msg.author.to_string(),
            timestamp: msg.timestamp,
        });
    }
    return js_messages;
}

#[wasm_bindgen]
pub async fn send_message(server_id: u64, channel_id: u64, content: String) -> String {
    let client = crate::new_client();
    if let Some(server_key) = load_server_key(server_id).await {
        let cipher = ChaCha20Poly1305::from(server_key);
        let ciphertext = cipher.encrypt(content.as_bytes());
        let encrypted_hex = hex::encode(ciphertext.encode());
        let sent_tx = client.send_message(server_id, channel_id, encrypted_hex).await;
        return sent_tx.tx_hash().to_string();
    }
    return String::new();
}

#[wasm_bindgen]
pub async fn delete_message(server_id: u64, channel_id: u64, message_id: u64) -> String {
    let client = crate::new_client();
    let sent_tx = client.delete_message(server_id, channel_id, message_id).await;
    let tx_hash = sent_tx.tx_hash().to_string();
    return tx_hash;
}

#[wasm_bindgen]
pub async fn kick_member(server_id: u64, target_hex: String) -> String {
    let client = crate::new_client();
    let target = crate::string_to_pubkey(&target_hex);
    let sent_tx = client.kick_member(server_id, target).await;
    let tx_hash = sent_tx.tx_hash().to_string();
    return tx_hash;
}

#[wasm_bindgen]
pub async fn get_server_key_hex(server_id: u64) -> Option<String> {
    let key = load_server_key(server_id).await?;
    let key_hex = hex::encode(key);
    return Some(key_hex);
}
