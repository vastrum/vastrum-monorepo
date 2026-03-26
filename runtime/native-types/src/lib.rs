mod kvbtree;
mod kvmap;
mod kvvec;
mod kvvecbtree;

pub use kvbtree::KvBTree;
pub use kvmap::KvMap;
pub use kvvec::KvVec;
pub use kvvecbtree::KvVecBTree;

use std::cell::RefCell;
use std::sync::Arc;

thread_local! {
    static DESER_CLIENT: RefCell<Option<Arc<vastrum_rpc_client::RpcClient>>> = const { RefCell::new(None) };
}

pub fn with_deser_client<T>(client: &Arc<vastrum_rpc_client::RpcClient>, f: impl FnOnce() -> T) -> T {
    DESER_CLIENT.with(|cell| {
        let prev = cell.borrow_mut().replace(client.clone());
        let result = f();
        *cell.borrow_mut() = prev;
        result
    })
}

pub(crate) fn get_deser_client() -> Arc<vastrum_rpc_client::RpcClient> {
    DESER_CLIENT.with(|cell| {
        cell.borrow().clone().expect("get_deser_client called outside with_deser_client")
    })
}
