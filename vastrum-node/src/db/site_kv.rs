//historykey, contains sitekv key but also height
//this allows rocksdb to sort by height
struct HistoryKey {
    storage_key: SiteKvStorageKey,
    height: u64,
}

impl HistoryKey {
    //in order to support rocksdb ordering on height need to_be_bytes
    fn encode(self) -> Vec<u8> {
        let mut buf = self.storage_key.encode();
        buf.extend_from_slice(&self.height.to_be_bytes());
        return buf;
    }
}

fn history_key(site_id: Sha256Digest, key: &str, height: u64) -> Vec<u8> {
    let storage_key = SiteKvStorageKey::new(site_id, key);
    let history_key = HistoryKey { storage_key, height }.encode();
    return history_key;
}

fn history_prefix_end(site_id: Sha256Digest, key: &str) -> Vec<u8> {
    history_key(site_id, key, u64::MAX)
}

struct ChangedKey {
    storage_key: SiteKvStorageKey,
    old_value: Vec<u8>,
}

impl Db {
    pub fn read_kv(&self, key: &str, site_id: Sha256Digest) -> Option<Vec<u8>> {
        self.get(cf::SITE_KV, SiteKvStorageKey::new(site_id, key).encode())
    }

    pub fn write_kv(&self, key: &str, value: Vec<u8>, site_id: Sha256Digest) {
        self.put(cf::SITE_KV, SiteKvStorageKey::new(site_id, key).encode(), value);
    }

    pub fn delete_kv(&self, key: &str, site_id: Sha256Digest) {
        self.delete(cf::SITE_KV, SiteKvStorageKey::new(site_id, key).encode());
    }

    pub fn read_kv_at_height(
        &self,
        key: &str,
        site_id: Sha256Digest,
        height: u64,
    ) -> Option<Vec<u8>> {
        let first_change_after = history_key(site_id, key, height + 1);
        let key_upper_bound = history_prefix_end(site_id, key);

        match self.seek_forward_bounded(cf::KV_HISTORY, &first_change_after, &key_upper_bound) {
            //if find entry in kv_history, that means key was changed after this height
            //either have case where key was empty at this height and then latest set
            //or key was empty before and should return that value
            Some(entry) => {
                let key_was_empty_at_this_height = entry.value.is_empty();
                if key_was_empty_at_this_height {
                    return None;
                } else {
                    return Some(entry.value);
                }
            }
            //no changes after this height, can return current value
            None => {
                return self.read_kv(key, site_id);
            }
        }
    }

    pub fn read_kv_with_proof(
        &self,
        key: &str,
        site_id: Sha256Digest,
        height: u64,
    ) -> Option<(Vec<u8>, StateProof)> {
        let value = self.read_kv_at_height(key, site_id, height)?;
        let sk = SiteKvStorageKey::new(site_id, key).encode();
        let proof = self.generate_state_proof(cf::SITE_KV, &sk, height)?;
        return Some((value, proof));
    }
}

impl BatchDb {
    pub fn write_kv(&self, key: &str, value: Vec<u8>, site_id: Sha256Digest) {
        self.put(cf::SITE_KV, SiteKvStorageKey::new(site_id, key).encode(), value);
    }

    pub fn delete_kv(&self, key: &str, site_id: Sha256Digest) {
        self.delete(cf::SITE_KV, SiteKvStorageKey::new(site_id, key).encode());
    }

    pub fn read_kv(&self, key: &str, site_id: Sha256Digest) -> Option<Vec<u8>> {
        self.get(cf::SITE_KV, SiteKvStorageKey::new(site_id, key).encode())
    }

    pub fn write_keyvalue_history_to_db(&self, block_height: u64, retention: u64) {
        let changed = self.collect_changed_keyvalues_this_batch();

        let mut history_keys = vec![];
        for entry in changed {
            let hk = HistoryKey { storage_key: entry.storage_key, height: block_height }.encode();
            history_keys.push(hk.clone());
            self.put(cf::KV_HISTORY, &hk, entry.old_value);
        }

        if !history_keys.is_empty() {
            self.put(
                cf::KV_HISTORY_PRUNE_INDEX,
                block_height.to_be_bytes(),
                borsh::to_vec(&history_keys).unwrap(),
            );
        }

        self.prune_kv_history(block_height, retention);
    }

    fn prune_kv_history(&self, block_height: u64, retention: u64) {
        let expired = block_height.saturating_sub(retention + 1);
        if expired == 0 {
            return;
        }
        let idx_key = expired.to_be_bytes();
        if let Some(data) = self.get(cf::KV_HISTORY_PRUNE_INDEX, idx_key) {
            if let Ok(old_keys) = borsh::from_slice::<Vec<Vec<u8>>>(&data) {
                for hk in old_keys {
                    self.delete(cf::KV_HISTORY, hk);
                }
            }
            self.delete(cf::KV_HISTORY_PRUNE_INDEX, idx_key);
        }
    }

    fn collect_changed_keyvalues_this_batch(&self) -> Vec<ChangedKey> {
        let state = self.state.lock();
        let mut changed = vec![];
        for cf_key in state.pending.keys() {
            if cf_key.cf != cf::SITE_KV {
                continue;
            }
            let old_value = self.db.get(cf::SITE_KV, &cf_key.key).unwrap_or_default();
            let storage_key: SiteKvStorageKey = borsh::from_slice(&cf_key.key).unwrap();
            changed.push(ChangedKey { storage_key, old_value });
        }
        return changed;
    }
}

use crate::db::{BatchDb, Db, cf};
use vastrum_shared_types::crypto::sha256::Sha256Digest;
use vastrum_shared_types::types::storage::SiteKvStorageKey;
use vastrum_shared_types::{borsh::BorshExt, types::rpc::types::StateProof};
