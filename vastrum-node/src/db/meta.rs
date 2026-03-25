use super::{BatchDb, Db, cf};
use vastrum_shared_types::borsh::BorshExt;

const LATEST_HEIGHT: &[u8] = b"latest_height";

impl Db {
    pub fn read_latest_finalized_height(&self) -> u64 {
        let bytes = self.get(cf::META, LATEST_HEIGHT);
        if let Some(bytes) = bytes {
            return u64::decode(&bytes).unwrap();
        } else {
            return 0;
        }
    }

    pub fn write_latest_height(&self, height: u64) {
        self.put(cf::META, LATEST_HEIGHT, height.encode());
    }
}

impl BatchDb {
    pub fn read_latest_finalized_height(&self) -> u64 {
        let bytes = self.get(cf::META, LATEST_HEIGHT);
        if let Some(bytes) = bytes {
            return u64::decode(&bytes).unwrap();
        } else {
            return 0;
        }
    }

    pub fn write_latest_height(&self, height: u64) {
        self.put(cf::META, LATEST_HEIGHT, height.encode());
    }
}
