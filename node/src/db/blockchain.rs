#[derive(Debug)]
pub struct BlockchainDatabase {
    db_connection: DBWithThreadMode<SingleThreaded>,
}
impl BlockchainDatabase {
    pub fn remove_db_if_exists(node_id: u16) {
        let db_path = &BlockchainDatabase::db_path(node_id);
        if std::fs::exists(db_path).unwrap() {
            std::fs::remove_dir_all(db_path).unwrap();
        }
    }
    pub fn db_path(node_id: u16) -> String {
        let db_path: String = format!("database/blockchaindb{node_id}");
        return db_path;
    }
    pub fn new() -> BlockchainDatabase {
        let node_id = std::env::var("NODE_ID").unwrap().parse::<u16>().unwrap();
        let db_path = &BlockchainDatabase::db_path(node_id);
        let db = DB::open_default(db_path.clone()).unwrap();

        let blockchain_database = BlockchainDatabase { db_connection: db };
        return blockchain_database;
    }
    pub fn write(&self, slot: SlotState) {
        let data = slot.encode();

        let mut height = 0;
        if let SlotState::Block(slot) = slot {
            height = slot.height;
        } else if let SlotState::Nullification(slot) = slot {
            height = slot.height;
        }
        self.db_connection.put(height.to_be_bytes(), data).unwrap();
    }
    pub fn read_slot(&self, height: u64) -> Option<SlotState> {
        let result = self.db_connection.get(height.to_be_bytes()).unwrap();

        if let Some(bytes) = result {
            if let Ok(data) = SlotState::decode(&bytes) {
                return Some(data);
            } else {
                println!("Failed to decode slot from database, problem");

                return None;
            }
        } else {
            return None;
        }
    }
}

use crate::consensus::types::SlotState;
use rocksdb::{DB, DBWithThreadMode, SingleThreaded};
use shared_types::borsh::BorshExt;
