#[wasm_bindgen]
pub async fn create_post(category_name: String, title: String, content: String) -> String {
    let concourse = ContractAbiClient::new(Sha256Digest::from_u64(0));
    let sent_tx = concourse.create_post(category_name, title, content).await;
    let tx_hash = sent_tx.tx_hash().to_string();
    return tx_hash;
}

#[wasm_bindgen]
pub async fn reply_to_post(category_name: String, post_id: u64, content: String) -> String {
    let concourse = ContractAbiClient::new(Sha256Digest::from_u64(0));
    let sent_tx = concourse.reply_to_post(category_name, post_id, content).await;
    let tx_hash = sent_tx.tx_hash().to_string();
    return tx_hash;
}
#[wasm_bindgen]
pub async fn create_category(name: String, description: String) -> String {
    let concourse = ContractAbiClient::new(Sha256Digest::from_u64(0));
    let sent_tx = concourse.create_category(name, description).await;
    let tx_hash = sent_tx.tx_hash().to_string();
    return tx_hash;
}

#[wasm_bindgen]
pub async fn get_all_categories() -> Vec<JSCategory> {
    let concourse = ContractAbiClient::new(Sha256Digest::from_u64(0));
    let state = concourse.state().await;

    let mut categories = vec![];
    for name in &state.category_list {
        if let Some(cat) = state.categories.get(name).await {
            let post_count = cat.posts.length().await;
            let latest_posts = cat.posts.get_descending_entries(1, 0).await;
            let latest_activity = latest_posts.first().map_or(0, |p| p.last_bump_time);
            categories.push(JSCategory {
                name: cat.name,
                description: cat.description,
                post_count,
                latest_activity,
            });
        }
    }
    return categories;
}

#[wasm_bindgen]
pub async fn get_category_posts(category_name: String, limit: usize, offset: usize) -> Vec<JSPost> {
    let concourse = ContractAbiClient::new(Sha256Digest::from_u64(0));
    let state = concourse.state().await;
    let category = state.categories.get(&category_name).await.unwrap();
    let posts = category.posts.get_descending_entries(limit, offset).await;
    let mut js_posts = Vec::with_capacity(posts.len());
    for post in &posts {
        let reply_count = post.replies.length().await;
        js_posts.push(convert_post(post, reply_count));
    }
    return js_posts;
}

#[wasm_bindgen]
pub async fn get_category_post_count(category_name: String) -> u64 {
    let concourse = ContractAbiClient::new(Sha256Digest::from_u64(0));
    let state = concourse.state().await;
    let category = state.categories.get(&category_name).await.unwrap();
    let count = category.posts.length().await;
    return count;
}

#[wasm_bindgen]
pub async fn get_post(category_name: String, post_id: u64) -> Option<JSPost> {
    let concourse = ContractAbiClient::new(Sha256Digest::from_u64(0));
    let state = concourse.state().await;
    let category = state.categories.get(&category_name).await.unwrap();
    let Some(post) = category.posts.get(post_id).await else { return None };
    let reply_count = post.replies.length().await;
    return Some(convert_post(&post, reply_count));
}

use serde::Serialize;
use vastrum_shared_types::crypto::sha256::Sha256Digest;
use tsify::Tsify;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub async fn delete_post(category_name: String, post_id: u64) -> String {
    let concourse = ContractAbiClient::new(Sha256Digest::from_u64(0));
    let sent_tx = concourse.delete_post(category_name, post_id).await;
    let tx_hash = sent_tx.tx_hash().to_string();
    return tx_hash;
}

#[wasm_bindgen]
pub async fn delete_reply(category_name: String, post_id: u64, reply_id: u64) -> String {
    let concourse = ContractAbiClient::new(Sha256Digest::from_u64(0));
    let sent_tx = concourse.delete_reply(category_name, post_id, reply_id).await;
    let tx_hash = sent_tx.tx_hash().to_string();
    return tx_hash;
}

#[wasm_bindgen]
pub async fn get_post_replies(
    category_name: String,
    post_id: u64,
    limit: usize,
    offset: usize,
) -> Vec<JSPostReply> {
    let concourse = ContractAbiClient::new(Sha256Digest::from_u64(0));
    let state = concourse.state().await;
    let category = state.categories.get(&category_name).await.unwrap();
    let post = category.posts.get(post_id).await.unwrap();
    let replies = post.replies.get_ascending_entries(limit, offset).await;
    let js_replies = replies
        .iter()
        .map(|reply| JSPostReply {
            id: reply.id,
            content: reply.content.clone(),
            timestamp: reply.timestamp,
            from: reply.from.to_string(),
        })
        .collect();
    return js_replies;
}

#[wasm_bindgen]
pub async fn get_moderators() -> Vec<String> {
    let concourse = ContractAbiClient::new(Sha256Digest::from_u64(0));
    let state = concourse.state().await;
    let moderators = state.admins.iter().map(|admin| admin.to_string()).collect();
    return moderators;
}

#[wasm_bindgen]
pub async fn get_my_public_key() -> String {
    let key = vastrum_frontend_lib::get_pub_key().await;
    let public_key_hex = hex::encode(key.to_bytes());
    return public_key_hex;
}

#[derive(Serialize, Tsify)]
#[tsify(into_wasm_abi)]
pub struct JSPost {
    pub id: u64,
    pub from: String,
    pub title: String,
    pub content: String,
    pub timestamp: u64,
    pub last_bump_time: u64,
    pub reply_count: u64,
}
#[derive(Serialize, Tsify)]
#[tsify(into_wasm_abi)]
pub struct JSPostReply {
    pub id: u64,
    pub from: String,
    pub content: String,
    pub timestamp: u64,
}

#[derive(Serialize, Tsify)]
#[tsify(into_wasm_abi)]
pub struct JSCategory {
    pub name: String,
    pub description: String,
    pub post_count: u64,
    pub latest_activity: u64,
}

fn convert_post(post: &Post, reply_count: u64) -> JSPost {
    JSPost {
        id: post.id,
        title: post.title.clone(),
        content: post.content.clone(),
        timestamp: post.timestamp,
        last_bump_time: post.last_bump_time,
        reply_count,
        from: post.from.to_string(),
    }
}

pub use concourse_abi::*;
