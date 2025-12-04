pub struct ComponentDatabase {
    node_id: u16,
}
impl ComponentDatabase {
    pub fn remove_db_if_exists(node_id: u16) {
        let db_path = &ComponentDatabase::db_path(node_id);
        if std::fs::exists(db_path).unwrap() {
            std::fs::remove_dir_all(db_path).unwrap();
        }
    }
    pub fn db_path(node_id: u16) -> String {
        let db_path: String = format!("database/componentdb{node_id}");
        return db_path;
    }

    pub fn init(&self) {
        let db_path = &ComponentDatabase::db_path(self.node_id);
        let _db = DB::open_default(db_path.clone()).unwrap();
    }
    pub fn new() -> ComponentDatabase {
        let node_id = std::env::var("NODE_ID").unwrap().parse::<u16>().unwrap();
        let db = ComponentDatabase { node_id: node_id };
        db.init();
        return db;
    }
    pub fn write(&self, wasm_component: WasmComponent) {
        let db_path = ComponentDatabase::db_path(self.node_id);
        let db = DB::open_default(db_path).unwrap();
        db.put(wasm_component.key.encode(), wasm_component.data).unwrap();
    }
    pub fn read_component(&self, key: Sha256Digest) -> Option<Vec<u8>> {
        let db_path: String = ComponentDatabase::db_path(self.node_id);
        let db_opts = Options::default();
        let db = DB::open_for_read_only(&db_opts, db_path, false).unwrap();

        let result = db.get(key.encode()).unwrap();

        if let Some(bytes) = result {
            return Some(bytes);
        } else {
            return None;
        }
    }
}

use crate::application::wasm_component_data::WasmComponent;
use rocksdb::{DB, Options};
use shared_types::{borsh::BorshExt, crypto::sha256::Sha256Digest};
