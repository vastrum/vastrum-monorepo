use super::{BatchDb, Db, cf};
use crate::execution::types::sitedata::SiteData;
use vastrum_shared_types::{borsh::BorshExt, crypto::sha256::Sha256Digest};

impl Db {
    pub fn read_site(&self, site_id: Sha256Digest) -> Option<SiteData> {
        let key = site_id.encode();
        let res = self.get(cf::SITE, key);
        if let Some(res) = res {
            return Some(SiteData::decode(&res).unwrap());
        } else {
            return None;
        }
    }

    pub fn write_site(&self, site_data: SiteData) {
        let key = site_data.site_id.encode();
        let value = site_data.encode();
        self.put(cf::SITE, key, value);
    }
}

impl BatchDb {
    pub fn read_site(&self, site_id: Sha256Digest) -> Option<SiteData> {
        let key = site_id.encode();
        let res = self.get(cf::SITE, key);
        if let Some(res) = res {
            return Some(SiteData::decode(&res).unwrap());
        } else {
            return None;
        }
    }

    pub fn write_site(&self, site_data: SiteData) {
        let key = site_data.site_id.encode();
        let value = site_data.encode();
        self.put(cf::SITE, key, value);
    }
}
