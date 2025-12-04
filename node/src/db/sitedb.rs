pub struct SiteDatabase {
    node_id: u16,
}
impl SiteDatabase {
    pub fn remove_db_if_exists(node_id: u16) {
        let db_path = &SiteDatabase::db_path(node_id);
        if std::fs::exists(db_path).unwrap() {
            std::fs::remove_dir_all(db_path).unwrap();
        }
    }
    pub fn db_path(node_id: u16) -> String {
        let db_path: String = format!("database/sitedb{node_id}");
        return db_path;
    }

    pub fn init(&self) {
        let db_path = &SiteDatabase::db_path(self.node_id);
        let _db = DB::open_default(db_path.clone()).unwrap();
    }
    pub fn new() -> SiteDatabase {
        let node_id = std::env::var("NODE_ID").unwrap().parse::<u16>().unwrap();
        let db = SiteDatabase { node_id };
        db.init();
        return db;
    }
    pub fn write(&self, site_data: SiteData) {
        let db_path = SiteDatabase::db_path(self.node_id);
        let db = DB::open_default(db_path).unwrap();

        let data = site_data.encode();
        db.put(site_data.site_id.encode(), data).unwrap();
    }
    pub fn read_site_data(&self, site_id: Sha256Digest) -> Option<SiteData> {
        let db_path: String = SiteDatabase::db_path(self.node_id);
        let db_opts = Options::default();
        let db = DB::open_for_read_only(&db_opts, db_path, false).unwrap();

        let result = db.get(site_id.encode()).unwrap();

        if let Some(bytes) = result {
            return Some(SiteData::decode(&bytes).unwrap());
        } else {
            return None;
        }
    }
}

use crate::application::sitedata::SiteData;
use rocksdb::{DB, Options};
use shared_types::{borsh::BorshExt, crypto::sha256::Sha256Digest};
