pub async fn start_http_server(
    db: Arc<Db>,
    networking: Arc<Networking>,
    rpc_nodes: Vec<RpcNodeEndpoint>,
    epoch_state: &EpochState,
) -> eyre::Result<()> {
    let addr = format!("0.0.0.0:{HTTP_RPC_PORT}");
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    let rpc = RPCHttpServer::new(networking, db);
    let app = rpc.router(rpc_nodes, epoch_state).await;

    axum::serve(listener, app).await?;
    Ok(())
}

impl RPCHttpServer {
    async fn router(&self, rpc_nodes: Vec<RpcNodeEndpoint>, epoch_state: &EpochState) -> Router {
        let cors = CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any);

        let frontend = frontend_builder::build_frontend(rpc_nodes, epoch_state).await;
        let state = AppState { networking: self.networking.clone(), db: self.db.clone(), frontend };
        return Router::new()
            .route("/submit/", post(RPCHttpServer::submit_transaction))
            .route("/page/", post(RPCHttpServer::get_page))
            .route("/getlatestblockheight/", get(RPCHttpServer::get_latest_block_height))
            .route("/getkeyvalue/", post(RPCHttpServer::get_key_value))
            .route("/getsiteidisdeployed/", post(RPCHttpServer::get_site_id_is_deployed))
            .route("/gettxhashinclusionstate/", post(RPCHttpServer::get_tx_hash_inclusion_state))
            .route("/resolvedomain/", post(RPCHttpServer::resolve_domain))
            .route("/ethexecutionrpc", any(RPCHttpServer::eth_execution_rpc))
            .route("/ethexecutionrpc/{*path}", any(RPCHttpServer::eth_execution_rpc))
            .route("/ethconsensusrpc", any(RPCHttpServer::eth_consensus_rpc))
            .route("/ethconsensusrpc/{*path}", any(RPCHttpServer::eth_consensus_rpc))
            .route("/rpchttpfallback/", post(RPCHttpServer::borsh_rpc))
            .route("/health", get(StatusCode::OK))
            .fallback(RPCHttpServer::serve_frontend)
            .layer(DefaultBodyLimit::max(MAX_RPC_BODY_SIZE))
            .layer(CompressionLayer::new())
            .layer(cors)
            .with_state(state);
    }

    async fn submit_transaction(
        State(state): State<AppState>,
        axum::Json(input): axum::Json<SubmitTransactionPayload>,
    ) -> impl IntoResponse {
        handlers::submit(&state.networking, input);
        StatusCode::OK
    }
    async fn get_page(
        State(state): State<AppState>,
        axum::Json(input): axum::Json<GetPagePayload>,
    ) -> impl IntoResponse {
        Json(handlers::get_page(&state.db, input))
    }
    async fn get_latest_block_height(State(state): State<AppState>) -> impl IntoResponse {
        Json(handlers::get_latest_block_height(&state.db))
    }
    async fn get_key_value(
        State(state): State<AppState>,
        axum::Json(input): axum::Json<GetKeyValuePayload>,
    ) -> impl IntoResponse {
        Json(handlers::get_key_value(&state.db, input))
    }

    async fn get_site_id_is_deployed(
        State(state): State<AppState>,
        axum::Json(input): axum::Json<GetSiteIDIsDeployed>,
    ) -> impl IntoResponse {
        Json(handlers::get_site_id_is_deployed(&state.db, input))
    }

    async fn get_tx_hash_inclusion_state(
        State(state): State<AppState>,
        axum::Json(input): axum::Json<GetTxHashIsIncluded>,
    ) -> impl IntoResponse {
        Json(handlers::get_tx_hash_inclusion_state(&state.db, input))
    }
    async fn resolve_domain(
        State(state): State<AppState>,
        axum::Json(input): axum::Json<ResolveDomainRequest>,
    ) -> impl IntoResponse {
        Json(handlers::resolve_domain(&state.db, input))
    }
    async fn borsh_rpc(
        State(state): State<AppState>,
        body: axum::body::Bytes,
    ) -> impl IntoResponse {
        let Ok(request) = borsh::from_slice::<RpcRequest>(&body) else {
            return StatusCode::BAD_REQUEST.into_response();
        };
        let result = route(&request, &state.db, &state.networking).await;
        match result {
            Some(rpc_body) => {
                let response = RpcResponse { id: request.id, body: rpc_body };
                (
                    [(axum::http::header::CONTENT_TYPE, "application/octet-stream")],
                    response.encode(),
                )
                    .into_response()
            }
            None => StatusCode::OK.into_response(),
        }
    }
    async fn eth_execution_rpc(method: Method, headers: HeaderMap, request: Request) -> Response {
        super::eth_proxy::eth_execution_rpc(method, headers, request).await
    }
    async fn eth_consensus_rpc(method: Method, headers: HeaderMap, request: Request) -> Response {
        super::eth_proxy::eth_consensus_rpc(method, headers, request).await
    }
    async fn serve_frontend(
        State(state): State<AppState>,
        headers: HeaderMap,
        uri: axum::http::Uri,
    ) -> Response {
        let path = uri.path().trim_start_matches('/');
        let accepts_brotli = client_accepts_brotli_compression(&headers);

        // try static asset first (css, js, wasm, ico)
        if let Some(response) = serve_static_asset(&state, path, accepts_brotli) {
            return response;
        }

        //if does not match asset serve index.html for all other paths
        serve_index_html(&state, accepts_brotli)
    }
    pub fn new(networking: Arc<Networking>, db: Arc<Db>) -> RPCHttpServer {
        return RPCHttpServer { networking, db };
    }
}

fn client_accepts_brotli_compression(headers: &HeaderMap) -> bool {
    headers
        .get(header::ACCEPT_ENCODING)
        .and_then(|v| v.to_str().ok())
        .is_some_and(|v| v.contains("br"))
}

fn serve_index_html(state: &AppState, accepts_brotli: bool) -> Response {
    if accepts_brotli {
        (
            [
                (header::CONTENT_TYPE, "text/html".to_string()),
                (header::CONTENT_ENCODING, "br".to_string()),
                (header::CACHE_CONTROL, "no-store".to_string()),
            ],
            (*state.frontend.compressed_html).clone(),
        )
            .into_response()
    } else {
        (
            [(header::CACHE_CONTROL, "no-store".to_string())],
            axum::response::Html(state.frontend.html.clone()),
        )
            .into_response()
    }
}

fn serve_static_asset(state: &AppState, path: &str, accepts_brotli: bool) -> Option<Response> {
    let (data, mime) = state.frontend.compressed_assets.get(path)?;
    if accepts_brotli {
        Some(
            (
                [
                    (header::CONTENT_TYPE, mime.clone()),
                    (header::CONTENT_ENCODING, "br".to_string()),
                ],
                data.clone(),
            )
                .into_response(),
        )
    } else {
        let content = frontend_builder::FrontendAssets::get(path)?;
        Some(([(header::CONTENT_TYPE, mime.clone())], content.data.into_owned()).into_response())
    }
}

pub struct RPCHttpServer {
    networking: Arc<Networking>,
    db: Arc<Db>,
}
#[derive(Clone)]
struct AppState {
    networking: Arc<Networking>,
    db: Arc<Db>,
    frontend: frontend_builder::Frontend,
}

use super::frontend_builder;
use crate::{
    consensus::validator_state_machine::EpochState,
    db::Db,
    p2p::networking::Networking,
    rpc::{handlers, webrtc_direct::router::route},
};
use axum::{
    Json, Router,
    extract::{DefaultBodyLimit, Request, State},
    http::{HeaderMap, StatusCode, header},
    response::{IntoResponse, Response},
    routing::{any, get, post},
};
use reqwest::Method;
use std::sync::Arc;
use tower_http::compression::CompressionLayer;
use tower_http::cors::{Any, CorsLayer};
use vastrum_shared_types::borsh::BorshExt;
use vastrum_shared_types::frontend::frontend_data::RpcNodeEndpoint;
use vastrum_shared_types::types::rpc::types::{
    GetKeyValuePayload, GetPagePayload, GetSiteIDIsDeployed, GetTxHashIsIncluded,
    ResolveDomainRequest, RpcRequest, RpcResponse, SubmitTransactionPayload,
};
use vastrum_shared_types::{limits::MAX_RPC_BODY_SIZE, ports::HTTP_RPC_PORT};
