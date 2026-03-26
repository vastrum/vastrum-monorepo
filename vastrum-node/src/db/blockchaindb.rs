use super::{BatchDb, Db, cf};
use crate::consensus::types::FinalizedBlock;
use vastrum_shared_types::borsh::BorshExt;

impl Db {
    pub fn read_block(&self, height: u64) -> Option<FinalizedBlock> {
        let key = height.encode();
        let res = self.get(cf::BLOCKCHAIN, key);
        if let Some(res) = res {
            return Some(FinalizedBlock::decode(&res).unwrap());
        } else {
            return None;
        }
    }

    pub fn write_block(&self, finalize: FinalizedBlock) {
        let height = finalize.block.height;
        let key = height.encode();
        let value = finalize.encode();
        self.put(cf::BLOCKCHAIN, key, value);
    }
}

impl BatchDb {
    pub fn read_block(&self, height: u64) -> Option<FinalizedBlock> {
        let key = height.encode();
        let res = self.get(cf::BLOCKCHAIN, key);
        if let Some(res) = res {
            return Some(FinalizedBlock::decode(&res).unwrap());
        } else {
            return None;
        }
    }

    pub fn write_block(&self, finalize: FinalizedBlock) {
        let height = finalize.block.height;
        let key = height.encode();
        let value = finalize.encode();
        self.put(cf::BLOCKCHAIN, key, value);
    }
}
