pub struct StateTree {
    latest_state_root: Sha256Digest,
}

impl StateTree {
    pub fn new() -> Self {
        StateTree { latest_state_root: Sha256Digest::default() }
    }

    pub fn restore(db: &Db) -> Self {
        StateTree { latest_state_root: db.read_jmt_root().unwrap_or_default() }
    }

    pub fn latest_state_root(&self) -> Sha256Digest {
        self.latest_state_root
    }

    pub fn write_state_updates_to_jmt_proof_db(
        &mut self,
        batch_db: &Arc<BatchDb>,
        block_height: u64,
    ) {
        self.apply_jmt_updates(batch_db, block_height);
        batch_db.prune_jmt_stale(block_height, KV_RETENTION_WINDOW);
    }

    fn apply_jmt_updates(&mut self, batch_db: &Arc<BatchDb>, block_height: u64) {
        let updates = batch_db.collect_jmt_updates();
        let jmt = Sha256Jmt::new(batch_db.inner_db());
        let (root, batch) = jmt.put_value_set(updates, block_height).unwrap();
        batch_db.write_jmt_update_to_db(
            &batch.node_batch,
            &batch.stale_node_index_batch,
            Sha256Digest::from(root.0),
            block_height,
        );
        self.latest_state_root = Sha256Digest::from(root.0);
    }
}

use crate::db::{BatchDb, Db};
use jmt::Sha256Jmt;
use vastrum_shared_types::crypto::sha256::Sha256Digest;
use vastrum_shared_types::limits::KV_RETENTION_WINDOW;
use std::sync::Arc;
