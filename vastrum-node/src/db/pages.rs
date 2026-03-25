fn page_key(site_id: Sha256Digest, path: &str) -> Vec<u8> {
    PageStorageKey::new(site_id, path).encode()
}

impl Db {
    pub fn read_page(&self, site_id: Sha256Digest, path: &str) -> Option<Page> {
        let key = page_key(site_id, path);
        let res = self.get(cf::PAGE, key);
        if let Some(res) = res {
            return Some(Page::decode(&res).unwrap());
        } else {
            return None;
        }
    }

    pub fn write_page(&self, page: Page) {
        let key = page_key(page.site_id, &page.path);
        self.put(cf::PAGE, key, page.encode());
    }
}

impl BatchDb {
    pub fn read_page(&self, site_id: Sha256Digest, page_path: &str) -> Option<Page> {
        let key = page_key(site_id, page_path);
        let res = self.get(cf::PAGE, key);
        if let Some(res) = res {
            return Some(Page::decode(&res).unwrap());
        } else {
            return None;
        }
    }

    pub fn write_page(&self, page: Page) {
        let key = page_key(page.site_id, &page.path);
        self.put(cf::PAGE, key, page.encode());
    }
}

use super::{BatchDb, Db, cf};
use vastrum_shared_types::types::storage::Page;
use vastrum_shared_types::types::storage::PageStorageKey;
use vastrum_shared_types::{borsh::BorshExt, crypto::sha256::Sha256Digest};
