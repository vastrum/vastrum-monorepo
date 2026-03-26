#[cfg(not(madsim))]
#[derive(rust_embed::RustEmbed)]
#[folder = "../web-client/app/dist/"]
pub struct FrontendAssets;

#[derive(Clone)]
pub struct Frontend {
    pub html: String,
    pub compressed_html: Arc<Vec<u8>>,
    pub compressed_assets: Arc<HashMap<String, (Vec<u8>, String)>>,
}

/// Build the frontend HTML (with injected frontend data) and pre-compress all assets with brotli.
pub async fn build_frontend(rpc_nodes: Vec<RpcNodeEndpoint>, epoch_state: &EpochState) -> Frontend {
    let (html, compressed_html) = build_index_html(rpc_nodes, epoch_state).await;
    let compressed_assets = brotli_compress_static_assets();
    Frontend { html, compressed_html, compressed_assets }
}

async fn build_index_html(
    rpc_nodes: Vec<RpcNodeEndpoint>,
    epoch_state: &EpochState,
) -> (String, Arc<Vec<u8>>) {
    let file = FrontendAssets::get("index.html").unwrap();
    let raw_html = String::from_utf8_lossy(&file.data);
    let helios_checkpoint = fetch_finalized_checkpoint().await;
    let html = inject_frontend_data(&raw_html, rpc_nodes, helios_checkpoint, epoch_state);
    let compressed_html = Arc::new(brotli_compress(html.as_bytes()));
    (html, compressed_html)
}

fn inject_frontend_data(
    html: &str,
    rpc_nodes: Vec<RpcNodeEndpoint>,
    helios_checkpoint: String,
    epoch_state: &EpochState,
) -> String {
    let mut genesis_validators = HashMap::new();
    for v in epoch_state.validator_data.values() {
        genesis_validators.insert(
            v.validator_index,
            ValidatorInfo {
                validator_index: v.validator_index,
                pub_key: v.pub_key.to_bytes(),
                stake: v.stake,
            },
        );
    }
    let total_validator_stake = epoch_state.total_validator_stake;

    let frontend_data =
        FrontendData { rpc_nodes, helios_checkpoint, genesis_validators, total_validator_stake };
    let encoded = serde_json::to_string(&frontend_data).unwrap();
    html.replace("</head>", &format!(r#"<script type="application/json" id="__frontendData">{encoded}</script></head>"#))
}

fn brotli_compress_static_assets() -> Arc<HashMap<String, (Vec<u8>, String)>> {
    let mut assets: HashMap<String, (Vec<u8>, String)> = HashMap::new();
    for path in FrontendAssets::iter() {
        let is_index = &*path == "index.html";
        if is_index {
            continue;
        }
        if let Some(file) = FrontendAssets::get(&path) {
            let mime = mime_guess::from_path(&*path).first_or_octet_stream().to_string();
            assets.insert(path.to_string(), (brotli_compress(&file.data), mime));
        }
    }
    Arc::new(assets)
}

use super::helios_checkpoint::fetch_finalized_checkpoint;
use crate::consensus::validator_state_machine::EpochState;
use vastrum_shared_types::compression::brotli::brotli_compress;
use vastrum_shared_types::frontend::frontend_data::{FrontendData, RpcNodeEndpoint, ValidatorInfo};
use std::collections::HashMap;
use std::sync::Arc;
