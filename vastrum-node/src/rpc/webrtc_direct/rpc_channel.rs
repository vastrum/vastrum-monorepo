pub struct RpcChannel {
    pub reader: vastrum_webrtc_direct_server::FramedReader,
    pub writer: FramedWriter,
    pub db: Arc<Db>,
    pub networking: Arc<Networking>,
}
impl RpcChannel {
    pub async fn run(mut self) {
        while let Some(request_bytes) = self.reader.recv().await {
            if request_bytes.len() > MAX_RPC_BODY_SIZE {
                continue;
            }
            let Ok(request) = borsh::from_slice::<RpcRequest>(&request_bytes) else {
                continue;
            };

            let writer = self.writer.clone();
            let db = self.db.clone();
            let networking = self.networking.clone();
            tokio::spawn(async move {
                Self::handle_request(request, writer, &db, &networking).await;
            });
        }
    }

    async fn handle_request(
        request: RpcRequest,
        writer: FramedWriter,
        db: &Db,
        networking: &Networking,
    ) {
        let Some(body) = router::route(&request, db, networking).await else {
            return;
        };
        let response = RpcResponse { id: request.id, body };

        let _ = writer.send(&response.encode()).await;
    }
}

use super::router;
use crate::{db::Db, p2p::networking::Networking};
use vastrum_shared_types::{
    borsh::BorshExt,
    limits::MAX_RPC_BODY_SIZE,
    types::rpc::types::{RpcRequest, RpcResponse},
};
use std::sync::Arc;
use vastrum_webrtc_direct_server::FramedWriter;
