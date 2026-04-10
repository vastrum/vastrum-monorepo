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
use vastrum_git_lib::ContractAbiClient;

#[derive(serde::Deserialize)]
pub struct InfoRefsParams {
    service: Option<String>,
}

pub fn router(contract: Arc<ContractAbiClient>) -> Router {
    Router::new()
        .route("/{repo}/info/refs", get(info_refs))
        .route("/{repo}/git-upload-pack", post(upload_pack))
        .with_state(contract)
}

/// Materialize a temp bare repo from chain for the given repo name.
/// Returns the TempDir (must be kept alive) and the bare repo path.
async fn materialize_temp_repo(
    contract: &ContractAbiClient,
    repo: &str,
) -> Result<(tempfile::TempDir, std::path::PathBuf), String> {
    let site_id = contract.site_id();
    let repo_owned = repo.to_string();

    let tmp = tempfile::tempdir().map_err(|e| format!("failed to create temp dir: {}", e))?;
    let bare_path = tmp.path().join(format!("{}.git", repo));

    let handle = tokio::runtime::Handle::current();
    let bare_path_clone = bare_path.clone();

    tokio::task::spawn_blocking(move || {
        let contract = ContractAbiClient::new(site_id);
        handle.block_on(async {
            crate::materialize::materialize_bare_repo(
                &bare_path_clone,
                &repo_owned,
                &contract,
            )
            .await
        })
    })
    .await
    .map_err(|e| format!("spawn_blocking failed: {}", e))?
    .map_err(|e| format!("repo '{}': {}", repo, e))?;

    Ok((tmp, bare_path))
}

/// GET /:repo/info/refs?service=git-upload-pack
async fn info_refs(
    State(contract): State<Arc<ContractAbiClient>>,
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

    let (_tmp, bare_path) = match materialize_temp_repo(&contract, &repo).await {
        Ok(r) => r,
        Err(e) => return (StatusCode::NOT_FOUND, e).into_response(),
    };

    let output = match Command::new("git")
        .args(["upload-pack", "--stateless-rpc", "--advertise-refs"])
        .arg(&bare_path)
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
    State(contract): State<Arc<ContractAbiClient>>,
    Path(repo): Path<String>,
    body: Bytes,
) -> Response {
    let (_tmp, bare_path) = match materialize_temp_repo(&contract, &repo).await {
        Ok(r) => r,
        Err(e) => return (StatusCode::NOT_FOUND, e).into_response(),
    };

    let mut child = match Command::new("git")
        .args(["upload-pack", "--stateless-rpc"])
        .arg(&bare_path)
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
