// Generate bindings of the guest and host components.
bindgen!("hostbindings" in "../webassembly/component-bindings.wit");

impl runtimebindings::Host for HostState {
    fn registerstaticroute(
        &mut self,
        route: wasmtime::component::__internal::String,
        html: wasmtime::component::__internal::String,
    ) -> () {
        println!("static route registered route: {route}");
        println!("self.site_id {:#?}", self.site_id);
        let page = Page { site_id: self.site_id, path: route, content: html };
        self.page_database.write(page);
    }
    fn blocktime(&mut self) -> u64 {
        let timestamp_seconds = SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs();
        return timestamp_seconds;
    }
    fn dbquery(
        &mut self,
        json: wasmtime::component::__internal::String,
    ) -> wasmtime::component::__internal::String {
        println!("db query {json}");
        let query = parse_query_json(&json);
        if let Ok(query) = query {
            let connection = rusqlite::Connection::open(self.db_path()).unwrap();
            let result = query_db(query, &self.site_id.to_string(), &connection);
            if let Ok(result) = result {
                if let Ok(result_json) = serde_json::to_string(&result) {
                    println!("successful return {result_json}");
                    return result_json;
                } else {
                    println!("could not serialize db query response to json");
                    return "err".to_string();
                }
            } else {
                println!("db query not succcesfull");
                return "err".to_string();
            }
        } else {
            println!("db query could not parse json");
            return "err".to_string();
        }
    }
    fn dbcreatetable(&mut self, json: wasmtime::component::__internal::String) {
        let database_create_table = parse_create_table_json(&json);
        if let Err(err) = database_create_table {
            println!("error when creating db:  {err:?}");
        } else if let Ok(database_create_table) = database_create_table {
            let connection = rusqlite::Connection::open(self.db_path()).unwrap();
            create_table(database_create_table, &self.site_id.to_string(), &connection).unwrap();
        }
    }
    fn dbinsertentry(&mut self, json: wasmtime::component::__internal::String) {
        println!("{json:#?}");
        let database_insert_entry = parse_insert_entry(&json).unwrap();
        println!("{database_insert_entry:#?}");
        let connection = rusqlite::Connection::open(self.db_path()).unwrap();
        insert_entry(database_insert_entry, &self.site_id.to_string(), &connection).unwrap();
    }
    fn dbupdateentry(&mut self, json: wasmtime::component::__internal::String) {
        let database_update_entry = parse_update_entry(&json).unwrap();
        let connection = rusqlite::Connection::open(self.db_path()).unwrap();
        update_entry(database_update_entry, &self.site_id.to_string(), &connection).unwrap();
    }
}
pub struct HostState {
    site_id: Sha256Digest,
    page_database: Arc<PageDatabase>,
    pub limits: StoreLimits,
}
impl HostState {
    fn db_path(&self) -> String {
        return sql_db_path();
    }
    pub fn new(
        site_id: Sha256Digest,
        page_database: Arc<PageDatabase>,
        limits: StoreLimits,
    ) -> HostState {
        return HostState { site_id: site_id, page_database: page_database, limits: limits };
    }
}

use crate::{
    application::{
        page::Page,
        sql::{
            db::{create_table, insert_entry, query_db, sql_db_path, update_entry},
            host_communication::{
                parse_create_table_json, parse_insert_entry, parse_query_json, parse_update_entry,
            },
        },
    },
    db::pagedb::PageDatabase,
};
use shared_types::crypto::sha256::Sha256Digest;
use std::{sync::Arc, time::SystemTime};
use wasmtime::{StoreLimits, component::bindgen};
