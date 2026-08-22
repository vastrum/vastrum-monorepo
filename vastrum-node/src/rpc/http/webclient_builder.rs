#[derive(Clone)]
pub struct WebClient {
    pub html: String,
    pub compressed_html: Arc<Vec<u8>>,
    pub compressed_assets: Arc<HashMap<String, (Vec<u8>, String)>>,
}

/// Build the web-client HTML (with injected web-client data) and pre-compress all assets with brotli.
pub async fn build_webclient(rpc_nodes: Vec<RpcNodeEndpoint>, epoch_state: &EpochState) -> WebClient {
    let (html, compressed_html) = build_index_html(rpc_nodes, epoch_state).await;
    let compressed_assets = brotli_compress_static_assets();
    WebClient { html, compressed_html, compressed_assets }
}

async fn build_index_html(
    rpc_nodes: Vec<RpcNodeEndpoint>,
    epoch_state: &EpochState,
) -> (String, Arc<Vec<u8>>) {
    let file = embedded_asset("index.html").unwrap();
    let raw_html = String::from_utf8_lossy(file.data);
    let helios_checkpoint = fetch_finalized_checkpoint().await;
    let html = inject_webclient_data(&raw_html, rpc_nodes, helios_checkpoint, epoch_state);
    let compressed_html = Arc::new(brotli_compress(html.as_bytes()));
    (html, compressed_html)
}

fn inject_webclient_data(
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

    let webclient_data =
        WebClientData { rpc_nodes, helios_checkpoint, genesis_validators, total_validator_stake };
    let encoded = serde_json::to_string(&webclient_data).unwrap();
    html.replace("</head>", &format!(r#"<script type="application/json" id="__webclientData">{encoded}</script></head>"#))
}

fn brotli_compress_static_assets() -> Arc<HashMap<String, (Vec<u8>, String)>> {
    let mut assets: HashMap<String, (Vec<u8>, String)> = HashMap::new();
    for asset in WEB_CLIENT_ASSETS {
        let is_index = asset.path == "index.html";
        if is_index {
            continue;
        }
        let mime = mime_guess::from_path(asset.path).first_or_octet_stream().to_string();
        assets.insert(asset.path.to_string(), (brotli_compress(asset.data), mime));
    }
    Arc::new(assets)
}

use super::embedded_assets::{WEB_CLIENT_ASSETS, embedded_asset};
use super::helios_checkpoint::fetch_finalized_checkpoint;
use crate::consensus::validator_state_machine::EpochState;
use vastrum_shared_types::compression::brotli::brotli_compress;
use vastrum_shared_types::webclient::webclient_data::{RpcNodeEndpoint, ValidatorInfo, WebClientData};
use std::collections::HashMap;
use std::sync::Arc;
