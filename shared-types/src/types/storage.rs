#[allow(unused_imports)]
use crate::borsh::*;
use crate::crypto::sha256::{Sha256Digest, sha256_hash};
use borsh::{BorshDeserialize, BorshSerialize};

#[derive(BorshSerialize, BorshDeserialize, Clone)]
pub struct SiteKvStorageKey {
    pub site_id: Sha256Digest,
    pub key_hash: Sha256Digest,
}

impl SiteKvStorageKey {
    pub fn new(site_id: Sha256Digest, key: &str) -> Self {
        Self { site_id, key_hash: sha256_hash(key.as_bytes()) }
    }
}

#[derive(BorshSerialize, BorshDeserialize, Clone)]
pub struct PageStorageKey {
    pub site_id: Sha256Digest,
    pub path_hash: Sha256Digest,
}

impl PageStorageKey {
    pub fn new(site_id: Sha256Digest, path: &str) -> Self {
        Self { site_id, path_hash: sha256_hash(path.as_bytes()) }
    }
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Eq, PartialEq, Clone)]
pub struct Page {
    pub site_id: Sha256Digest,
    pub path: String,
    pub brotli_html_content: Vec<u8>,
}

#[derive(BorshSerialize)]
pub struct JmtKeyInput<'a> {
    pub cf_namespace: u8,
    pub key: &'a [u8],
}

pub fn cf_to_namespace_byte(cf: &str) -> u8 {
    match cf {
        "site" => 0,
        "sitekv" => 1,
        "domain" => 2,
        "page" => 3,
        other => panic!("unknown state CF in JMT namespace mapping: {other}"),
    }
}
