/// Route a WebRTC RPC request to the appropriate handler.
/// Returns `Some(body)` to send a response, `None` for fire-and-forget.
pub async fn route(request: &RpcRequest, db: &Db, networking: &Networking) -> Option<RpcBody> {
    match request.route.as_str() {
        "page" => {
            let Ok(payload) = borsh::from_slice::<GetPagePayload>(&request.body) else {
                return Some(RpcBody::Error("invalid payload".into()));
            };
            let page = handlers::get_page(db, payload);
            return Some(RpcBody::Success(page.encode()));
        }
        "getlatestblockheight" => {
            let height = handlers::get_latest_block_height(db);
            return Some(RpcBody::Success(height.encode()));
        }
        "getkeyvalue" => {
            let Ok(payload) = borsh::from_slice::<GetKeyValuePayload>(&request.body) else {
                return Some(RpcBody::Error("invalid payload".into()));
            };
            let value = handlers::get_key_value(db, payload);
            return Some(RpcBody::Success(value.encode()));
        }
        "submit" => {
            let Ok(payload) = borsh::from_slice::<SubmitTransactionPayload>(&request.body) else {
                return Some(RpcBody::Error("invalid payload".into()));
            };
            handlers::submit(networking, payload);
            return None;
        }
        "getsiteidisdeployed" => {
            let Ok(payload) = borsh::from_slice::<GetSiteIDIsDeployed>(&request.body) else {
                return Some(RpcBody::Error("invalid payload".into()));
            };
            let deployed = handlers::get_site_id_is_deployed(db, payload);
            return Some(RpcBody::Success(deployed.encode()));
        }
        "gettxhashinclusionstate" => {
            let Ok(payload) = borsh::from_slice::<GetTxHashIsIncluded>(&request.body) else {
                return Some(RpcBody::Error("invalid payload".into()));
            };
            let inclusion = handlers::get_tx_hash_inclusion_state(db, payload);
            return Some(RpcBody::Success(inclusion.encode()));
        }
        "resolvedomain" => {
            let Ok(payload) = borsh::from_slice::<ResolveDomainRequest>(&request.body) else {
                return Some(RpcBody::Error("invalid payload".into()));
            };
            let resolved = handlers::resolve_domain(db, payload);
            return Some(RpcBody::Success(resolved.encode()));
        }
        "ethproxy" => {
            let Ok(payload) = borsh::from_slice::<EthProxyRequest>(&request.body) else {
                return Some(RpcBody::Error("invalid payload".into()));
            };
            let resp = if let Some((_, path)) = payload.url.split_once("/ethexecutionrpc") {
                handlers::eth_execution_proxy(path, &payload.method, payload.body).await
            } else if let Some((_, path)) = payload.url.split_once("/ethconsensusrpc") {
                handlers::eth_consensus_proxy(path, &payload.method, payload.body).await
            } else {
                unreachable!("unknown eth proxy endpoint: {}", payload.url)
            };
            return Some(RpcBody::Success(resp.encode()));
        }
        _ => {
            return Some(RpcBody::Error(format!("unknown route: {}", request.route)));
        }
    }
}

use crate::{db::Db, p2p::networking::Networking, rpc::handlers};
use vastrum_shared_types::{
    borsh::BorshExt,
    types::rpc::types::{
        EthProxyRequest, GetKeyValuePayload, GetPagePayload, GetSiteIDIsDeployed,
        GetTxHashIsIncluded, ResolveDomainRequest, RpcBody, RpcRequest, SubmitTransactionPayload,
    },
};
