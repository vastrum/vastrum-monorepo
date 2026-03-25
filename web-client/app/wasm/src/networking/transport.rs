pub struct RpcTransport {
    pub inner: Rc<FramedClient>,
    pub pending: Rc<RefCell<HashMap<u64, oneshot::Sender<RpcResponse>>>>,
    pub next_id: Cell<u64>,
}

impl RpcTransport {
    pub fn send_request(&self, route: &str, body: &[u8]) -> Result<oneshot::Receiver<RpcResponse>> {
        let id = self.next_id.get();
        self.next_id.set(id + 1);

        let (tx, rx) = oneshot::channel();
        self.pending.borrow_mut().insert(id, tx);

        let req = RpcRequest { id, route: route.to_string(), body: body.to_vec() };
        let encoded = req.encode();
        if encoded.len() > vastrum_shared_types::limits::MAX_RPC_BODY_SIZE {
            return Err(crate::utils::error::WasmErr::PayloadTooLarge);
        }
        self.inner.send(&encoded)?;

        Ok(rx)
    }

    pub fn send_fire_and_forget(&self, route: &str, body: &[u8]) -> Result<()> {
        let id = self.next_id.get();
        self.next_id.set(id + 1);
        let req = RpcRequest { id, route: route.to_string(), body: body.to_vec() };
        let encoded = req.encode();
        if encoded.len() > vastrum_shared_types::limits::MAX_RPC_BODY_SIZE {
            return Err(crate::utils::error::WasmErr::PayloadTooLarge);
        }
        self.inner.send(&encoded)?;
        Ok(())
    }
}

use crate::utils::error::Result;
use futures::channel::oneshot;
use vastrum_shared_types::borsh::BorshExt;
use vastrum_shared_types::types::rpc::types::{RpcRequest, RpcResponse};
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::Rc;
use webrtc_direct_client::FramedClient;
