mod blockchaindb;

pub mod cf {
    pub const SITE: &str = "site";
    pub const DOMAIN: &str = "domain";
    pub const MODULE: &str = "module";
    pub const SITE_KV: &str = "sitekv";
    pub const BLOCKCHAIN: &str = "blockchain";
    pub const INCLUDED_TXS: &str = "included_txs";
    pub const PAGE: &str = "page";
    pub const META: &str = "meta";
    pub const VOTE_STATE: &str = "vote_state";
    pub const JMT_NODES: &str = "jmt_nodes";
    pub const JMT_VALUES: &str = "jmt_values";
    pub const JMT_STALE: &str = "jmt_stale";
    pub const KV_HISTORY: &str = "kv_history";
    pub const KV_HISTORY_PRUNE_INDEX: &str = "kv_history_index";
}

pub struct Db {
    #[cfg(not(madsim))]
    rocks: Arc<rocksdb::DB>,
    #[cfg(madsim)]
    mem: Mutex<HashMap<CfKey, Vec<u8>>>,
    data_path: PathBuf,
}

#[cfg(not(madsim))]
impl Db {
    pub fn default_path() -> PathBuf {
        std::env::var("VASTRUM_DATA_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| dirs::data_dir().unwrap().join("vastrum"))
    }

    pub fn new() -> Db {
        Self::open_fresh(Self::default_path())
    }

    pub fn open(path: impl Into<PathBuf>) -> Db {
        use rocksdb::{
            BlockBasedOptions, Cache, ColumnFamilyDescriptor, DB, DBCompressionType,
            DataBlockIndexType, Options,
        };

        let path = path.into();
        let cache = Cache::new_lru_cache(256 * 1024 * 1024);

        let mut block_opts = BlockBasedOptions::default();
        block_opts.set_block_cache(&cache);
        block_opts.set_bloom_filter(10.0, false);
        block_opts.set_cache_index_and_filter_blocks(true);
        block_opts.set_pin_l0_filter_and_index_blocks_in_cache(true);
        block_opts.set_data_block_index_type(DataBlockIndexType::BinaryAndHash);
        block_opts.set_data_block_hash_ratio(0.75);

        let mut cf_opts = Options::default();
        cf_opts.set_block_based_table_factory(&block_opts);
        cf_opts.set_compression_type(DBCompressionType::Lz4);

        let mut db_opts = Options::default();
        db_opts.create_if_missing(true);
        db_opts.create_missing_column_families(true);
        db_opts.set_max_open_files(-1);
        db_opts.set_advise_random_on_open(true);
        db_opts.set_max_background_jobs(8);
        db_opts.set_compaction_readahead_size(4 * 1024 * 1024);
        db_opts.set_level_compaction_dynamic_level_bytes(true);
        db_opts.set_enable_pipelined_write(true);
        db_opts.set_use_fsync(false);

        let cfs = vec![
            ColumnFamilyDescriptor::new(cf::SITE, cf_opts.clone()),
            ColumnFamilyDescriptor::new(cf::DOMAIN, cf_opts.clone()),
            ColumnFamilyDescriptor::new(cf::MODULE, cf_opts.clone()),
            ColumnFamilyDescriptor::new(cf::SITE_KV, cf_opts.clone()),
            ColumnFamilyDescriptor::new(cf::BLOCKCHAIN, cf_opts.clone()),
            ColumnFamilyDescriptor::new(cf::INCLUDED_TXS, cf_opts.clone()),
            ColumnFamilyDescriptor::new(cf::PAGE, cf_opts.clone()),
            ColumnFamilyDescriptor::new(cf::META, cf_opts.clone()),
            ColumnFamilyDescriptor::new(cf::VOTE_STATE, cf_opts.clone()),
            ColumnFamilyDescriptor::new(cf::JMT_NODES, cf_opts.clone()),
            ColumnFamilyDescriptor::new(cf::JMT_VALUES, cf_opts.clone()),
            ColumnFamilyDescriptor::new(cf::JMT_STALE, cf_opts.clone()),
            ColumnFamilyDescriptor::new(cf::KV_HISTORY, cf_opts.clone()),
            ColumnFamilyDescriptor::new(cf::KV_HISTORY_PRUNE_INDEX, cf_opts),
        ];

        Db {
            rocks: Arc::new(DB::open_cf_descriptors(&db_opts, &path, cfs).unwrap()),
            data_path: path,
        }
    }

    pub fn open_fresh(path: impl Into<PathBuf>) -> Db {
        let path = path.into();
        if path.exists() {
            let _ = std::fs::remove_dir_all(&path);
        }
        Self::open(path)
    }

    pub fn put(&self, cf: &str, key: impl AsRef<[u8]>, value: Vec<u8>) {
        self.rocks.put_cf(self.rocks.cf_handle(cf).unwrap(), key, value).unwrap();
    }

    pub fn get(&self, cf: &str, key: impl AsRef<[u8]>) -> Option<Vec<u8>> {
        self.rocks.get_cf(self.rocks.cf_handle(cf).unwrap(), key).unwrap()
    }

    pub fn delete(&self, cf: &str, key: impl AsRef<[u8]>) {
        self.rocks.delete_cf(self.rocks.cf_handle(cf).unwrap(), key).unwrap();
    }

    pub fn seek_prev(&self, cf: &str, key: &[u8]) -> Option<DbEntry> {
        let cf_handle = self.rocks.cf_handle(cf)?;
        let mut iter = self
            .rocks
            .iterator_cf(&cf_handle, rocksdb::IteratorMode::From(key, rocksdb::Direction::Reverse));
        let (key, value) = iter.next()?.ok()?;
        Some(DbEntry { key: key.to_vec(), value: value.to_vec() })
    }

    /// Finds first entry in cf with key sort value above key and <= upper_bound
    pub fn seek_forward_bounded(
        &self,
        cf: &str,
        key: &[u8],
        upper_bound: &[u8],
    ) -> Option<DbEntry> {
        let cf_handle = self.rocks.cf_handle(cf)?;
        let mut iter = self
            .rocks
            .iterator_cf(&cf_handle, rocksdb::IteratorMode::From(key, rocksdb::Direction::Forward));
        let (found_key, value) = iter.next()?.ok()?;
        if &*found_key <= upper_bound {
            Some(DbEntry { key: found_key.to_vec(), value: value.to_vec() })
        } else {
            None
        }
    }

    pub fn write_batch(&self, pending_writes: HashMap<CfKey, Vec<u8>>, deletes: &[CfKey]) {
        let mut wb = rocksdb::WriteBatch::default();
        for (CfKey { cf, key }, value) in pending_writes {
            wb.put_cf(self.rocks.cf_handle(&cf).unwrap(), key, value);
        }
        for CfKey { cf, key } in deletes {
            wb.delete_cf(self.rocks.cf_handle(cf).unwrap(), key);
        }
        self.rocks.write(wb).unwrap();
    }
}

#[cfg(madsim)]
impl Db {
    pub fn default_path() -> PathBuf {
        PathBuf::from("/tmp/vastrum-madsim")
    }
    pub fn new() -> Db {
        Db { mem: Mutex::new(HashMap::new()), data_path: Self::default_path() }
    }
    pub fn open(path: impl Into<PathBuf>) -> Db {
        Db { mem: Mutex::new(HashMap::new()), data_path: path.into() }
    }
    pub fn open_fresh(path: impl Into<PathBuf>) -> Db {
        Db { mem: Mutex::new(HashMap::new()), data_path: path.into() }
    }
    pub fn put(&self, cf: &str, key: impl AsRef<[u8]>, value: Vec<u8>) {
        let key = CfKey::new(cf, key.as_ref());
        self.mem.lock().insert(key, value);
    }

    pub fn get(&self, cf: &str, key: impl AsRef<[u8]>) -> Option<Vec<u8>> {
        let key = CfKey::new(cf, key.as_ref());
        self.mem.lock().get(&key).cloned()
    }

    pub fn delete(&self, cf: &str, key: impl AsRef<[u8]>) {
        let key = CfKey::new(cf, key.as_ref());
        self.mem.lock().remove(&key);
    }

    pub fn seek_prev(&self, cf: &str, key: &[u8]) -> Option<DbEntry> {
        let mem = self.mem.lock();
        mem.iter()
            .filter(|(k, _)| k.cf == cf && k.key.as_slice() <= key)
            .max_by(|(a, _), (b, _)| a.key.cmp(&b.key))
            .map(|(k, v)| DbEntry { key: k.key.clone(), value: v.clone() })
    }

    pub fn seek_forward_bounded(
        &self,
        cf: &str,
        key: &[u8],
        upper_bound: &[u8],
    ) -> Option<DbEntry> {
        let mem = self.mem.lock();
        mem.iter()
            .filter(|(k, _)| {
                k.cf == cf && k.key.as_slice() >= key && k.key.as_slice() <= upper_bound
            })
            .min_by(|(a, _), (b, _)| a.key.cmp(&b.key))
            .map(|(k, v)| DbEntry { key: k.key.clone(), value: v.clone() })
    }

    pub fn write_batch(&self, pending_writes: HashMap<CfKey, Vec<u8>>, deletes: &[CfKey]) {
        let mut mem = self.mem.lock();
        for (cf_key, value) in pending_writes {
            mem.insert(cf_key, value);
        }
        for cf_key in deletes {
            mem.remove(cf_key);
        }
    }
}

impl Db {
    pub fn compiled_modules_dir(&self) -> PathBuf {
        self.data_path.join("compiled_modules")
    }
}

pub struct DbEntry {
    pub key: Vec<u8>,
    pub value: Vec<u8>,
}

#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct CfKey {
    pub cf: String,
    pub key: Vec<u8>,
}

impl CfKey {
    fn new(cf: &str, key: &[u8]) -> Self {
        CfKey { cf: cf.to_string(), key: key.to_vec() }
    }
}
pub enum PendingOp {
    Write(Vec<u8>),
    Delete,
}

type PendingOps = BTreeMap<CfKey, PendingOp>;

enum WriteMode {
    Direct,
    Revertable(PendingOps),
}

struct BatchState {
    pending: PendingOps,
    write_mode: WriteMode,
}

impl BatchState {
    fn get_revertable(&self, cf: &str, key: &[u8]) -> Option<&PendingOp> {
        let WriteMode::Revertable(rw) = &self.write_mode else { return None };
        rw.get(&CfKey::new(cf, key))
    }

    fn get_pending(&self, cf: &str, key: &[u8]) -> Option<&PendingOp> {
        self.pending.get(&CfKey::new(cf, key))
    }
}

pub struct BatchDb {
    db: Arc<Db>,
    state: Mutex<BatchState>,
}

impl BatchDb {
    pub fn new(db: Arc<Db>) -> Arc<Self> {
        Arc::new(BatchDb {
            db,
            state: Mutex::new(BatchState {
                pending: BTreeMap::new(),
                write_mode: WriteMode::Direct,
            }),
        })
    }

    //3 different layers of keys with different priority
    //in order to support revertable state for transactions who panic
    //and block execution db disk writes should be revertable
    pub fn get(&self, cf: &str, key: impl AsRef<[u8]>) -> Option<Vec<u8>> {
        let key = key.as_ref();
        let state = self.state.lock();
        //first check if pending revertable tx this key (highest priority)
        if let Some(op) = state.get_revertable(cf, key) {
            return match op {
                PendingOp::Write(v) => Some(v.clone()),
                PendingOp::Delete => None,
            };
        }
        //then check if this pending batchdb has the key
        if let Some(op) = state.get_pending(cf, key) {
            return match op {
                PendingOp::Write(v) => Some(v.clone()),
                PendingOp::Delete => None,
            };
        }
        drop(state);
        //then check if underlying rocksdb has key
        self.db.get(cf, key)
    }

    pub fn put(&self, cf: &str, key: impl AsRef<[u8]>, value: Vec<u8>) {
        let cf_key = CfKey::new(cf, key.as_ref());
        let mut state = self.state.lock();
        match &mut state.write_mode {
            WriteMode::Revertable(revertable_ops) => {
                revertable_ops.insert(cf_key, PendingOp::Write(value));
            }
            WriteMode::Direct => {
                state.pending.insert(cf_key, PendingOp::Write(value));
            }
        }
    }

    pub fn delete(&self, cf: &str, key: impl AsRef<[u8]>) {
        let cf_key = CfKey::new(cf, key.as_ref());
        let mut state = self.state.lock();
        match &mut state.write_mode {
            WriteMode::Revertable(revertable_ops) => {
                revertable_ops.insert(cf_key, PendingOp::Delete);
            }
            WriteMode::Direct => {
                state.pending.insert(cf_key, PendingOp::Delete);
            }
        }
    }

    //Write this batch to disk atomically (no partial writes, crash safe)
    pub fn commit(&self) {
        let mut state = self.state.lock();
        let ops = std::mem::take(&mut state.pending);
        drop(state);

        let mut writes = HashMap::new();
        let mut deletes = Vec::new();
        for (key, op) in ops {
            match op {
                PendingOp::Write(v) => {
                    writes.insert(key, v);
                }
                PendingOp::Delete => deletes.push(key),
            }
        }
        self.db.write_batch(writes, &deletes);
    }

    pub fn begin_revertable(&self) {
        let mut state = self.state.lock();
        let no_active_revertable_tx = matches!(state.write_mode, WriteMode::Direct);
        debug_assert!(no_active_revertable_tx);
        state.write_mode = WriteMode::Revertable(BTreeMap::new());
    }

    pub fn commit_revertable(&self) {
        let mut state = self.state.lock();
        let WriteMode::Revertable(rw) = std::mem::replace(&mut state.write_mode, WriteMode::Direct)
        else {
            return;
        };
        state.pending.extend(rw);
    }

    pub fn rollback_revertable(&self) {
        self.state.lock().write_mode = WriteMode::Direct;
    }

    pub fn inner_db(&self) -> &Db {
        &self.db
    }
}
mod domain;
mod included_txs;
pub mod jmt;
mod meta;
mod module;
mod pages;
pub mod round_state;
mod site;
mod site_kv;
pub mod vote_state;

use parking_lot::Mutex;
use std::collections::{BTreeMap, HashMap};
use std::path::PathBuf;
use std::sync::Arc;

#[cfg(test)]
#[path = "db_tests.rs"]
mod tests;
