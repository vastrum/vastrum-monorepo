pub struct DomainDatabase {
    node_id: u16,
}
impl DomainDatabase {
    pub fn remove_db_if_exists(node_id: u16) {
        let db_path = &DomainDatabase::db_path(node_id);
        if std::fs::exists(db_path).unwrap() {
            std::fs::remove_dir_all(db_path).unwrap();
        }
    }
    pub fn db_path(node_id: u16) -> String {
        let db_path: String = format!("database/domaindb{node_id}");
        return db_path;
    }

    pub fn init(&self) {
        let db_path = &DomainDatabase::db_path(self.node_id);
        let _db = DB::open_default(db_path.clone()).unwrap();
    }
    pub fn new() -> DomainDatabase {
        let node_id = std::env::var("NODE_ID").unwrap().parse::<u16>().unwrap();
        let db = DomainDatabase { node_id: node_id };
        db.init();
        return db;
    }
    pub fn write(&self, domain_data: DomainData) {
        let db_path = DomainDatabase::db_path(self.node_id);
        let db = DB::open_default(db_path).unwrap();

        let data = domain_data.encode();
        db.put(domain_data.domain_name, data).unwrap();
    }
    pub fn read_site_data(&self, domain_name: &str) -> Option<DomainData> {
        let db_path: String = DomainDatabase::db_path(self.node_id);
        let db_opts = Options::default();
        let db = DB::open_for_read_only(&db_opts, db_path, false).unwrap();

        let result = db.get(domain_name).unwrap();

        if let Some(bytes) = result {
            return Some(DomainData::decode(&bytes).unwrap());
        } else {
            return None;
        }
    }
}
use rocksdb::{DB, Options};
use shared_types::{borsh::BorshExt, types::application::domaindata::DomainData};
