use crate::repo_cache::RepoCache;
use axum::{
    body::Bytes,
    extract::{Path, Query, State},
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Router,
};
use std::sync::Arc;
use tokio::process::Command;

#[derive(serde::Deserialize)]
pub struct InfoRefsParams {
    service: Option<String>,
}

pub fn router(cache: Arc<RepoCache>) -> Router {
    Router::new()
        .route("/{repo}/info/refs", get(info_refs))
        .route("/{repo}/git-upload-pack", post(upload_pack))
        .with_state(cache)
}

/// GET /:repo/info/refs?service=git-upload-pack
async fn info_refs(
    State(cache): State<Arc<RepoCache>>,
    Path(repo): Path<String>,
    Query(params): Query<InfoRefsParams>,
) -> Response {
    let service = params.service.as_deref().unwrap_or("git-upload-pack");
    if service != "git-upload-pack" {
        return (
            StatusCode::FORBIDDEN,
            "only git-upload-pack is supported (read-only)",
        )
            .into_response();
    }

    let repo_path = match cache.ensure_fresh(&repo).await {
        Ok(path) => path,
        Err(e) => {
            return (StatusCode::NOT_FOUND, format!("repo '{}': {}", repo, e)).into_response()
        }
    };

    let output = match Command::new("git")
        .args(["upload-pack", "--stateless-rpc", "--advertise-refs"])
        .arg(&repo_path)
        .output()
        .await
    {
        Ok(o) => o,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("failed to spawn git: {}", e),
            )
                .into_response()
        }
    };

    if !output.status.success() {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!(
                "git upload-pack failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ),
        )
            .into_response();
    }

    // Wrap in smart HTTP protocol: pkt-line header + output + flush
    let mut body = Vec::new();
    let header_line = format!("# service={}\n", service);
    write_pkt_line(&mut body, header_line.as_bytes());
    body.extend_from_slice(b"0000"); // flush-pkt
    body.extend_from_slice(&output.stdout);

    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        format!("application/x-{}-advertisement", service)
            .parse()
            .unwrap(),
    );
    headers.insert(header::CACHE_CONTROL, "no-cache".parse().unwrap());

    (headers, body).into_response()
}

/// POST /:repo/git-upload-pack
async fn upload_pack(
    State(cache): State<Arc<RepoCache>>,
    Path(repo): Path<String>,
    body: Bytes,
) -> Response {
    let repo_path = match cache.ensure_fresh(&repo).await {
        Ok(path) => path,
        Err(e) => {
            return (StatusCode::NOT_FOUND, format!("repo '{}': {}", repo, e)).into_response()
        }
    };

    let mut child = match Command::new("git")
        .args(["upload-pack", "--stateless-rpc"])
        .arg(&repo_path)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
    {
        Ok(c) => c,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("failed to spawn git: {}", e),
            )
                .into_response()
        }
    };

    // Write request body to stdin
    {
        use tokio::io::AsyncWriteExt;
        let mut stdin = child.stdin.take().unwrap();
        if stdin.write_all(&body).await.is_err() {
            return (StatusCode::INTERNAL_SERVER_ERROR, "failed to write to git")
                .into_response();
        }
        drop(stdin);
    }

    let output = match child.wait_with_output().await {
        Ok(o) => o,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("git upload-pack failed: {}", e),
            )
                .into_response()
        }
    };

    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        "application/x-git-upload-pack-result".parse().unwrap(),
    );
    headers.insert(header::CACHE_CONTROL, "no-cache".parse().unwrap());

    (headers, output.stdout).into_response()
}

/// Write a pkt-line formatted line.
fn write_pkt_line(buf: &mut Vec<u8>, data: &[u8]) {
    let len = data.len() + 4;
    buf.extend_from_slice(format!("{:04x}", len).as_bytes());
    buf.extend_from_slice(data);
}
