const META_JMT_ROOT: &[u8] = b"jmt_root";
const JMT_TRACKED_CFS: [&str; 4] = ["site", "sitekv", "domain", "page"];

//key format: key_hash (32 bytes) + version (8 bytes BE)
fn jmt_value_key(key_hash: KeyHash, version: Version) -> Vec<u8> {
    [key_hash.0.as_slice(), &version.to_be_bytes()].concat()
}

impl TreeReader for Db {
    fn get_node_option(&self, node_key: &NodeKey) -> anyhow::Result<Option<Node>> {
        match self.get(cf::JMT_NODES, borsh::to_vec(node_key)?) {
            Some(bytes) => Ok(Some(borsh::from_slice(&bytes)?)),
            None => Ok(None),
        }
    }

    fn get_value_option(
        &self,
        max_version: Version,
        key_hash: KeyHash,
    ) -> anyhow::Result<Option<OwnedValue>> {
        let seek_key = jmt_value_key(key_hash, max_version);
        if let Some(entry) = self.seek_prev(cf::JMT_VALUES, &seek_key) {
            if entry.key.len() == 40 && entry.key[..32] == key_hash.0[..] {
                return Ok(Some(entry.value));
            }
        }
        return Ok(None);
    }

    fn get_rightmost_leaf(&self) -> anyhow::Result<Option<(NodeKey, LeafNode)>> {
        return Ok(None);
    }
}

impl BatchDb {
    pub fn collect_jmt_updates(&self) -> Vec<(KeyHash, Option<OwnedValue>)> {
        let state = self.state.lock();
        let mut updates = Vec::new();
        for (cf_key, op) in &state.pending {
            if !JMT_TRACKED_CFS.contains(&cf_key.cf.as_str()) {
                continue;
            }
            let jmt_key_input =
                JmtKeyInput { cf_namespace: cf_to_namespace_byte(&cf_key.cf), key: &cf_key.key };
            let encoded = borsh::to_vec(&jmt_key_input).unwrap();
            let key_hash = KeyHash::with::<Sha256>(&encoded);
            match op {
                PendingOp::Write(v) => {
                    updates.push((key_hash, Some(Sha256::digest(v).to_vec())));
                }
                PendingOp::Delete => {
                    updates.push((key_hash, None));
                }
            }
        }
        return updates;
    }

    pub fn write_jmt_update_to_db(
        &self,
        node_batch: &NodeBatch,
        stale_nodes: &jmt::storage::StaleNodeIndexBatch,
        root: Sha256Digest,
        version: u64,
    ) {
        self.persist_jmt_nodes(node_batch);
        self.persist_jmt_values(node_batch);
        self.persist_stale_node_keys(stale_nodes, version);
        self.put(cf::META, META_JMT_ROOT, root.to_bytes().into());
    }

    fn persist_jmt_nodes(&self, node_batch: &NodeBatch) {
        for (key, node) in node_batch.nodes() {
            self.put(cf::JMT_NODES, borsh::to_vec(key).unwrap(), borsh::to_vec(node).unwrap());
        }
    }

    fn persist_jmt_values(&self, node_batch: &NodeBatch) {
        for ((ver, key_hash), value) in node_batch.values() {
            if let Some(v) = value {
                self.put(cf::JMT_VALUES, jmt_value_key(*key_hash, *ver), v.clone());
            }
        }
    }

    fn persist_stale_node_keys(
        &self,
        stale_nodes: &jmt::storage::StaleNodeIndexBatch,
        version: u64,
    ) {
        let stale_keys: Vec<Vec<u8>> =
            stale_nodes.iter().map(|idx| borsh::to_vec(&idx.node_key).unwrap()).collect();
        if !stale_keys.is_empty() {
            self.put(
                cf::JMT_STALE,
                version.to_be_bytes().as_slice(),
                borsh::to_vec(&stale_keys).unwrap(),
            );
        }
    }

    pub fn prune_jmt_stale(&self, current_height: u64, retention: u64) {
        let expired = current_height.saturating_sub(retention + 1);
        if expired == 0 {
            return;
        }
        let key = expired.to_be_bytes();
        if let Some(data) = self.get(cf::JMT_STALE, key) {
            if let Ok(node_keys) = borsh::from_slice::<Vec<Vec<u8>>>(&data) {
                for nk in node_keys {
                    self.delete(cf::JMT_NODES, nk);
                }
            }
            self.delete(cf::JMT_STALE, key);
        }
    }
}

impl Db {
    pub fn read_jmt_root(&self) -> Option<Sha256Digest> {
        let bytes = self.get(cf::META, META_JMT_ROOT)?;
        Some(Sha256Digest::from(<[u8; 32]>::try_from(bytes.as_slice()).ok()?))
    }
}

#[cfg(not(madsim))]
impl Db {
    pub fn generate_state_proof(
        &self,
        cf: &str,
        key: &[u8],
        state_height: u64,
    ) -> Option<StateProof> {
        //state proof is delayed 1 block
        let block_height = state_height.checked_add(1)?;
        let jmt_version = state_height;

        let jmt_key = JmtKeyInput { cf_namespace: cf_to_namespace_byte(cf), key };
        let key_hash = KeyHash::with::<Sha256>(&borsh::to_vec(&jmt_key).unwrap());
        let jmt = Sha256Jmt::new(self);
        let (_stored_value, proof) = jmt.get_with_proof(key_hash, jmt_version).ok()?;

        let finalized = self.read_block(block_height)?;
        let state_root = finalized.block.previous_block_state_root;

        let block_header = BlockHeader {
            height: block_height,
            previous_block_hash: finalized.block.previous_block_hash,
            timestamp: finalized.block.timestamp,
            previous_block_state_root: state_root,
            transactions_hash: sha256_hash(&borsh::to_vec(&finalized.block.transactions).unwrap()),
        };

        let proof = StateProof {
            proof,
            block_header,
            round: finalized.round,
            finalization_votes: finalized.votes.into_iter().collect(),
        };
        return Some(proof);
    }
}

#[cfg(madsim)]
impl Db {
    pub fn generate_state_proof(
        &self,
        _cf: &str,
        _key: &[u8],
        _state_height: u64,
    ) -> Option<StateProof> {
        None
    }
}

use crate::db::{BatchDb, Db, PendingOp, cf};
use jmt::storage::{LeafNode, Node, NodeBatch, NodeKey, TreeReader};
use jmt::{KeyHash, OwnedValue, Sha256Jmt, Version};
use sha2::{Digest, Sha256};
use vastrum_shared_types::crypto::sha256::{Sha256Digest, sha256_hash};
use vastrum_shared_types::types::consensus::BlockHeader;
use vastrum_shared_types::types::rpc::types::StateProof;
use vastrum_shared_types::types::storage::{JmtKeyInput, cf_to_namespace_byte};
