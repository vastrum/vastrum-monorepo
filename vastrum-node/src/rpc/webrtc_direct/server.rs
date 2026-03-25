pub async fn start_webrtc_server(
    db: Arc<Db>,
    networking: Arc<Networking>,
    dtls_key: DtlsKey,
) -> eyre::Result<()> {
    let listen_addr: std::net::SocketAddr = format!("0.0.0.0:{WEBRTC_PORT}").parse().unwrap();
    let dtls_cert = dtls_key.to_dtls_cert();
    let mut server = WebRtcServer::bind(listen_addr, dtls_cert).await?;

    tokio::spawn(async move {
        while let Some((conn, _addr)) = server.accept().await {
            let (reader, writer) = conn.split();
            let channel =
                RpcChannel { reader, writer, db: db.clone(), networking: networking.clone() };
            tokio::spawn(channel.run());
        }
    });

    Ok(())
}

use super::rpc_channel::RpcChannel;
use crate::{db::Db, p2p::networking::Networking};
use vastrum_shared_types::ports::WEBRTC_PORT;
use std::sync::Arc;
use vastrum_webrtc_direct_server::{DtlsKey, WebRtcServer};
