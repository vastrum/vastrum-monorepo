pub async fn eth_execution_rpc(method: Method, _headers: HeaderMap, request: Request) -> Response {
    let (path, body) = match extract_path_and_body(request, "/ethexecutionrpc").await {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let resp = handlers::eth_execution_proxy(&path, method.as_str(), body).await;
    into_axum_response(resp)
}

pub async fn eth_consensus_rpc(method: Method, _headers: HeaderMap, request: Request) -> Response {
    let (path, body) = match extract_path_and_body(request, "/ethconsensusrpc").await {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let resp = handlers::eth_consensus_proxy(&path, method.as_str(), body).await;
    into_axum_response(resp)
}

async fn extract_path_and_body(
    request: Request,
    prefix: &str,
) -> Result<(String, Vec<u8>), Response> {
    let pq = request.uri().path_and_query().map(|pq| pq.as_str()).unwrap_or("/");
    let path = pq.strip_prefix(prefix).unwrap_or(pq).to_string();
    let body = axum::body::to_bytes(request.into_body(), MAX_PROXY_BODY_SIZE)
        .await
        .map(|b| b.to_vec())
        .map_err(|e| {
            (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to read request body: {e}"))
                .into_response()
        })?;
    Ok((path, body))
}

fn into_axum_response(resp: EthProxyResponse) -> Response {
    let status = StatusCode::from_u16(resp.status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
    (status, [(axum::http::header::CONTENT_TYPE, resp.content_type)], resp.body).into_response()
}

use crate::{rpc::handlers, utils::limits::MAX_PROXY_BODY_SIZE};
use axum::{
    extract::Request,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use reqwest::Method;
use vastrum_shared_types::types::rpc::types::EthProxyResponse;
