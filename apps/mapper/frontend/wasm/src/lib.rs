/// Fetch a single MVT tile from the contract KV store.
/// Returns the raw tile bytes, or empty vec if not found.
#[wasm_bindgen]
pub async fn get_tile(z: u8, x: u32, y: u32) -> Vec<u8> {
    let client = new_client();
    let state = client.state().await;
    let coord = TileCoord { z, x, y };
    return state.tiles.get(&coord).await.unwrap_or_default();
}

/// Fetch map metadata (bounds, zoom range).
#[wasm_bindgen]
pub async fn get_metadata() -> JSMapMetadata {
    let client = new_client();
    let state = client.state().await;
    let m = &state.metadata;
    return JSMapMetadata {
        min_zoom: m.min_zoom,
        max_zoom: m.max_zoom,
        center_lat: m.center_lat as f64 / 1_000_000.0,
        center_lng: m.center_lng as f64 / 1_000_000.0,
        bounds_min_lat: m.bounds_min_lat as f64 / 1_000_000.0,
        bounds_min_lng: m.bounds_min_lng as f64 / 1_000_000.0,
        bounds_max_lat: m.bounds_max_lat as f64 / 1_000_000.0,
        bounds_max_lng: m.bounds_max_lng as f64 / 1_000_000.0,
    };
}

fn new_client() -> ContractAbiClient {
    return ContractAbiClient::new(Sha256Digest::from([0u8; 32]));
}

#[derive(serde::Serialize, Tsify)]
#[tsify(into_wasm_abi)]
pub struct JSMapMetadata {
    pub min_zoom: u8,
    pub max_zoom: u8,
    pub center_lat: f64,
    pub center_lng: f64,
    pub bounds_min_lat: f64,
    pub bounds_min_lng: f64,
    pub bounds_max_lat: f64,
    pub bounds_max_lng: f64,
}

pub use mapper_abi::*;
use vastrum_shared_types::crypto::sha256::Sha256Digest;
use tsify::Tsify;
use wasm_bindgen::prelude::*;
