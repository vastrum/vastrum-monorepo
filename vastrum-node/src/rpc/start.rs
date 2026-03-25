#[cfg(not(madsim))]
pub fn start_rpc_node(
    db: Arc<Db>,
    networking: Arc<Networking>,
    dtls_key: DtlsKey,
    rpc_nodes: Vec<RpcNodeEndpoint>,
    epoch_state: EpochState,
) {
    tokio::spawn(async move {
        if let Err(e) = start_rpc_servers(db, networking, dtls_key, rpc_nodes, &epoch_state).await {
            eprintln!("RPC server failed: {e}");
            std::process::exit(1);
        }
    });
}

#[cfg(not(madsim))]
async fn start_rpc_servers(
    db: Arc<Db>,
    networking: Arc<Networking>,
    dtls_key: DtlsKey,
    rpc_nodes: Vec<RpcNodeEndpoint>,
    epoch_state: &EpochState,
) -> eyre::Result<()> {
    start_webrtc_server(db.clone(), networking.clone(), dtls_key).await?;
    start_http_server(db, networking, rpc_nodes, epoch_state).await?;
    Ok(())
}

#[cfg(madsim)]
pub fn start_rpc_node(
    _db: Arc<Db>,
    _networking: Arc<Networking>,
    _dtls_key: DtlsKey,
    _rpc_nodes: Vec<RpcNodeEndpoint>,
    _epoch_state: EpochState,
) {
}

#[cfg(not(madsim))]
use super::http::server::start_http_server;
#[cfg(not(madsim))]
use crate::rpc::webrtc_direct::server::start_webrtc_server;
use crate::{consensus::validator_state_machine::EpochState, db::Db, p2p::networking::Networking};
use vastrum_shared_types::frontend::frontend_data::RpcNodeEndpoint;
use std::sync::Arc;
use vastrum_webrtc_direct_server::DtlsKey;
