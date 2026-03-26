use super::{BatchDb, Db, cf};
use vastrum_shared_types::{borsh::BorshExt, crypto::sha256::Sha256Digest};

impl Db {
    pub fn check_tx_inclusion_state(&self, tx_hash: Sha256Digest) -> bool {
        let key = tx_hash.encode();
        self.get(cf::INCLUDED_TXS, key).is_some()
    }

    pub fn set_tx_as_included(&self, tx_hash: Sha256Digest) {
        let key = tx_hash.encode();
        self.put(cf::INCLUDED_TXS, key, vec![0]);
    }
}

impl BatchDb {
    pub fn check_tx_inclusion_state(&self, tx_hash: Sha256Digest) -> bool {
        let key = tx_hash.encode();
        self.get(cf::INCLUDED_TXS, key).is_some()
    }

    pub fn set_tx_as_included(&self, tx_hash: Sha256Digest) {
        let key = tx_hash.encode();
        self.put(cf::INCLUDED_TXS, key, vec![0]);
    }
}
