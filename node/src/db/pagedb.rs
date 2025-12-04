#[derive(Debug)]
pub struct PageDatabase {
    node_id: u16,
}
impl PageDatabase {
    pub fn remove_db_if_exists(node_id: u16) {
        let db_path = &PageDatabase::db_path(node_id);
        if std::fs::exists(db_path).unwrap() {
            std::fs::remove_dir_all(db_path).unwrap();
        }
    }

    pub fn db_path(node_id: u16) -> String {
        let db_path = format!("database/pagedb{node_id}");
        return db_path;
    }

    pub fn init(&self) {
        let db_path = &PageDatabase::db_path(self.node_id);
        let _db = DB::open_default(db_path.clone()).unwrap();
    }
    pub fn new() -> PageDatabase {
        let node_id = std::env::var("NODE_ID").unwrap().parse::<u16>().unwrap();
        let db = PageDatabase { node_id: node_id };
        db.init();
        return db;
    }
    fn get_page_key(site_id: Sha256Digest, page_path: String) -> String {
        let key = format!("{}|{}", site_id.to_string(), page_path);
        return key;
    }

    pub fn write(&self, page: Page) {
        let db_path = PageDatabase::db_path(self.node_id);
        let db = DB::open_default(db_path).unwrap();

        let data = page.encode();
        let key = PageDatabase::get_page_key(page.site_id, page.path);
        db.put(key, data).unwrap();
    }
    pub fn read_page(&self, site_id: Sha256Digest, page_path: String) -> Option<Page> {
        let db_path = PageDatabase::db_path(self.node_id);
        let db_opts = Options::default();
        let db = DB::open_for_read_only(&db_opts, db_path.clone(), false).unwrap();

        let key = PageDatabase::get_page_key(site_id, page_path);

        let result = db.get(key).unwrap();

        if let Some(bytes) = result {
            return Some(Page::decode(&bytes).expect("todo: error check"));
        } else {
            return None;
        }
    }
}

use crate::application::page::Page;
use rocksdb::{DB, Options};
use shared_types::{borsh::BorshExt, crypto::sha256::Sha256Digest};
