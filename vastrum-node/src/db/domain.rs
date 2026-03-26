use super::{BatchDb, Db, cf};
use vastrum_shared_types::{borsh::BorshExt, types::application::domaindata::DomainData};

impl Db {
    pub fn read_domain(&self, domain_name: &str) -> Option<DomainData> {
        let res = self.get(cf::DOMAIN, domain_name.to_string().encode());
        if let Some(res) = res {
            return Some(DomainData::decode(&res).unwrap());
        } else {
            return None;
        }
    }

    pub fn write_domain(&self, domain_data: DomainData) {
        let key = domain_data.domain_name.encode();
        let value = domain_data.encode();
        self.put(cf::DOMAIN, key, value);
    }
}

impl BatchDb {
    pub fn read_domain(&self, domain_name: &str) -> Option<DomainData> {
        let res = self.get(cf::DOMAIN, domain_name.to_string().encode());
        if let Some(res) = res {
            return Some(DomainData::decode(&res).unwrap());
        } else {
            return None;
        }
    }

    pub fn write_domain(&self, domain_data: DomainData) {
        let key = domain_data.domain_name.encode();
        let value = domain_data.encode();
        self.put(cf::DOMAIN, key, value);
    }
}
