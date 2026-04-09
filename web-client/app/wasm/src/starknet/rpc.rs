pub async fn send_starknet_rpc(req: StarknetRPCRequest) -> Value {
    super::worker::send_starknet_rpc_to_worker(req).await
}

use serde_json::Value;
use vastrum_shared_types::iframerpc::types::StarknetRPCRequest;
