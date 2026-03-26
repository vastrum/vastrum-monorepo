use crate::SentTxBehavior;
use crate::encryption::{get_dm_cipher, try_decrypt_content};
use crate::types::{JSDmSummary, JSMessage};
use vastrum_shared_types::borsh::BorshExt;
use vastrum_frontend_lib::get_pub_key;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub async fn send_dm(recipient_hex: String, content: String) -> String {
    let client = crate::new_client();
    let recipient = crate::string_to_pubkey(&recipient_hex);
    let encrypted_content = match get_dm_cipher(&recipient).await {
        Some(cipher) => {
            let ciphertext = cipher.encrypt(content.as_bytes());
            hex::encode(ciphertext.encode())
        }
        None => content,
    };
    let sent_tx = client.send_dm(recipient, encrypted_content).await;
    sent_tx.tx_hash().to_string()
}

#[wasm_bindgen]
pub async fn get_my_dms(count: u64, offset: u64) -> Vec<JSDmSummary> {
    let client = crate::new_client();
    let public_key = get_pub_key().await;
    let state = client.state().await;
    let profile = match state.user_profiles.get(&public_key).await {
        Some(p) => p,
        None => return vec![],
    };
    let dm_keys =
        profile.dm_activity.get_descending_entries(count as usize, offset as usize).await;
    let mut dms = Vec::new();
    for dm_key in &dm_keys {
        if let Some(convo) = state.dm_conversations.get(dm_key).await {
            let other = crate::dm_other(dm_key, public_key);
            let other_hex = hex::encode(other.to_bytes());
            let recent_messages = convo.messages.get_descending_entries(1, 0).await;
            let latest_message = recent_messages.first();
            let last_message = match latest_message {
                Some(msg) => {
                    let cipher = get_dm_cipher(&other).await;
                    Some(match &cipher {
                        Some(c) => try_decrypt_content(c, &msg.content),
                        None => msg.content.clone(),
                    })
                }
                None => None,
            };
            dms.push(JSDmSummary {
                other_user: other_hex,
                last_message,
                last_timestamp: latest_message.map(|msg| msg.timestamp),
                next_message_id: convo.messages.length().await,
            });
        }
    }
    dms
}

#[wasm_bindgen]
pub async fn get_dm_messages(other_user_hex: String, count: u64) -> Vec<JSMessage> {
    let client = crate::new_client();
    let public_key = get_pub_key().await;
    let other = crate::string_to_pubkey(&other_user_hex);
    let key = crate::make_dm_key(public_key, other);
    let state = client.state().await;
    let conversation = match state.dm_conversations.get(&key).await {
        Some(found_convo) => found_convo,
        None => return vec![],
    };
    let mut messages = conversation.messages.get_descending_entries(count as usize, 0).await;
    messages.reverse();
    let cipher = get_dm_cipher(&other).await;
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
