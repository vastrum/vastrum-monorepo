pub fn parse_query_json(data: &str) -> Result<DatabaseQuery> {
    let database_query: DatabaseQuery = serde_json::from_str(&data)?;
    Ok(database_query)
}

pub fn parse_create_table_json(data: &str) -> Result<DatabaseCreateTable> {
    let database_create_table: DatabaseCreateTable = serde_json::from_str(&data)?;
    Ok(database_create_table)
}

pub fn parse_insert_entry(data: &str) -> Result<DatabaseInsertEntry> {
    let database_insert_entry: DatabaseInsertEntry = serde_json::from_str(&data)?;
    Ok(database_insert_entry)
}

pub fn parse_update_entry(data: &str) -> Result<DatabaseUpdateEntry> {
    let databaseupdate_entry: DatabaseUpdateEntry = serde_json::from_str(&data)?;
    Ok(databaseupdate_entry)
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DatabaseQuery {
    pub table_id: u32,
    pub number_of_fields: u32,
    pub sorting_operations: Vec<SortingOperation>,
    pub where_operations: Vec<WhereOperation>,
    pub limit: u32,
    pub offset: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SortingOperation {
    pub field_id: u32,
    pub descending: bool, //descending or ascending direction in this sort
}

#[derive(Serialize, Deserialize, Debug)]
pub struct WhereOperation {
    pub field_id: u32,
    pub value: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DatabaseQueryResult {
    pub result: Vec<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DatabaseCreateTable {
    pub table_id: u32,
    pub fields: Vec<SQLFieldTypes>,
}

//https://serde.rs/enum-number.html
#[derive(Serialize_repr, Deserialize_repr, Debug, PartialEq)]
#[repr(u8)]
pub enum SQLFieldTypes {
    Integer,
    Text,
    Blob,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DatabaseInsertEntry {
    pub table_id: u32,
    pub data: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DatabaseUpdateEntry {
    pub table_id: u32,
    pub select_on_primary_key_value: String,
    pub data: Vec<String>,
}

use serde::{Deserialize, Serialize};
use serde_json::Result;
use serde_repr::*;

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::application::sql::host_communication::{
        DatabaseQuery, SQLFieldTypes, SortingOperation, WhereOperation, parse_create_table_json,
        parse_query_json,
    };

    #[test]
    fn test_parse_query_json() {
        let query = DatabaseQuery {
            number_of_fields: 3,
            table_id: 0,
            sorting_operations: vec![SortingOperation { field_id: 0, descending: true }],
            where_operations: vec![WhereOperation { field_id: 0, value: String::from("33") }],
            limit: 123,
            offset: 0,
        };
        let data = serde_json::to_string(&query).unwrap();

        let parsed_query = parse_query_json(&data).unwrap();

        assert!(parsed_query.table_id == query.table_id);
        assert!(parsed_query.limit == query.limit);
    }
    #[test]
    fn test_parse_create_table_json() {
        let data = json!({"table_id":0,"fields":[1,1,1,1,1]}).to_string();
        let parsed = parse_create_table_json(&data).unwrap();

        assert!(parsed.table_id == 0);
        assert!(
            parsed.fields
                == vec![
                    SQLFieldTypes::Text,
                    SQLFieldTypes::Text,
                    SQLFieldTypes::Text,
                    SQLFieldTypes::Text,
                    SQLFieldTypes::Text
                ]
        );
    }
}
