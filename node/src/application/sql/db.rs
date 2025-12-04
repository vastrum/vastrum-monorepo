pub fn query_db(
    query: DatabaseQuery,
    application_table_namespace: &str,
    connection: &rusqlite::Connection,
) -> Result<DatabaseQueryResult, rusqlite::Error> {
    let (sql_command, params) = Query::select(query.table_id, application_table_namespace)
        .sql_where(query.where_operations)
        .sql_sort_determistic(query.sorting_operations)
        .sql_limit(query.limit, query.offset)
        .calculate();
    let mut stmt = connection.prepare(&sql_command)?;
    let rows = stmt.query_map(rusqlite::params_from_iter(params), |row| {
        let mut res = vec![];
        let number_of_fields: usize = query.number_of_fields.try_into().unwrap();
        for i in 1..(number_of_fields + 1) {
            let string: String = row.get(i).unwrap();
            res.push(string);
        }
        Ok(res)
    })?;

    let mut filter_rows = vec![];
    for row in rows {
        if let Ok(value) = row {
            filter_rows.push(value);
        }
    }
    Ok(DatabaseQueryResult { result: filter_rows })
}
pub fn create_table(
    database_create_table: DatabaseCreateTable,
    application_table_namespace: &str,
    connection: &rusqlite::Connection,
) -> Result<(), rusqlite::Error> {
    let (sql_command, params) =
        Query::create_table(database_create_table, application_table_namespace);

    connection.execute(&sql_command, rusqlite::params_from_iter(params))?;
    Ok(())
}

pub fn insert_entry(
    database_insert_entry: DatabaseInsertEntry,
    application_table_namespace: &str,
    connection: &rusqlite::Connection,
) -> Result<(), rusqlite::Error> {
    let (sql_command, params) =
        Query::sql_insert(database_insert_entry, application_table_namespace);
    connection.execute(&sql_command, rusqlite::params_from_iter(params))?;
    Ok(())
}

pub fn update_entry(
    database_update_entry: DatabaseUpdateEntry,
    application_table_namespace: &str,
    connection: &rusqlite::Connection,
) -> Result<(), rusqlite::Error> {
    let (sql_command, params) =
        Query::sql_update(database_update_entry, application_table_namespace);
    connection.execute(&sql_command, rusqlite::params_from_iter(params))?;
    Ok(())
}

pub fn remove_db_if_exists_port() {
    let db_path = sql_db_path();
    if std::fs::exists(&db_path).unwrap() {
        std::fs::remove_file(db_path).unwrap();
    }
}
pub fn remove_db_if_exists(db_path: &str) -> Result<(), rusqlite::Error> {
    if std::fs::exists(db_path).unwrap() {
        std::fs::remove_file(db_path).unwrap();
    }

    Ok(())
}
//todo use this
pub fn _handle_application_query(
    query_json: &str,
    application_table_namespace: &str,
    connection: &rusqlite::Connection,
) -> Result<(), rusqlite::Error> {
    let Ok(query) = parse_query_json(query_json) else { return Ok(()) };

    const MAX_LIMIT: u32 = 1000;
    const MAX_SORTING_OPERATIONS: usize = 1;
    const MAX_SELECT_OPERATIONS: usize = 1;

    //sanitize
    //limit max 1000
    let limit = query.limit <= MAX_LIMIT;
    let below_max_sorting_operations = query.sorting_operations.len() <= MAX_SORTING_OPERATIONS;
    let below_max_where_operations = query.where_operations.len() <= MAX_SELECT_OPERATIONS;

    let valid_query = limit && below_max_sorting_operations && below_max_where_operations;

    if valid_query {
        query_db(query, application_table_namespace, connection)?;
    }
    Ok(())
}
pub fn sql_db_path() -> String {
    let node_id = std::env::var("NODE_ID").unwrap().parse::<u16>().unwrap();
    return format!("./database/app{}.db3", node_id);
}
#[cfg(test)]
mod tests {
    use crate::application::sql::{
        db::{create_table, insert_entry, query_db, remove_db_if_exists, update_entry},
        host_communication::{
            DatabaseCreateTable, DatabaseInsertEntry, DatabaseQuery, DatabaseUpdateEntry,
            SQLFieldTypes, SortingOperation, WhereOperation,
        },
    };

    #[test]
    fn test_create_db_insert_query() {
        let insert_1 = DatabaseInsertEntry {
            table_id: 0,
            data: vec!["a".to_string(), "654".to_string(), "\"asjd1j223\"".to_string()],
        };
        let insert_2 = DatabaseInsertEntry {
            table_id: 0,
            data: vec!["b".to_string(), "3455".to_string(), "\"asjd1j223\"".to_string()],
        };
        let insert_3 = DatabaseInsertEntry {
            table_id: 0,
            data: vec!["c".to_string(), "53".to_string(), "\"asjd1j223\"".to_string()],
        };
        let insert_4 = DatabaseInsertEntry {
            table_id: 0,
            data: vec!["d".to_string(), "123".to_string(), "\"asjd1j223\"".to_string()],
        };

        let query_sort = DatabaseQuery {
            number_of_fields: 3,
            table_id: 0,
            sorting_operations: vec![SortingOperation { field_id: 0, descending: true }],
            where_operations: vec![],
            limit: 2,
            offset: 0,
        };
        let query_sort_and_where = DatabaseQuery {
            number_of_fields: 3,
            table_id: 0,
            sorting_operations: vec![SortingOperation { field_id: 0, descending: true }],
            where_operations: vec![WhereOperation { field_id: 1, value: "123".to_string() }],
            limit: 2,
            offset: 0,
        };

        let database_create_table: DatabaseCreateTable = DatabaseCreateTable {
            table_id: 0,
            fields: vec![SQLFieldTypes::Text, SQLFieldTypes::Text, SQLFieldTypes::Text],
        };

        const DB_PATH: &str = "./test1.db3";
        remove_db_if_exists(DB_PATH).unwrap();

        let connection = rusqlite::Connection::open(DB_PATH).unwrap();
        let application_table_namespace = "firstcontract".to_string();
        create_table(database_create_table, &application_table_namespace, &connection).unwrap();
        insert_entry(insert_1, &application_table_namespace, &connection).unwrap();
        insert_entry(insert_2, &application_table_namespace, &connection).unwrap();
        insert_entry(insert_3, &application_table_namespace, &connection).unwrap();
        insert_entry(insert_4, &application_table_namespace, &connection).unwrap();

        let result_sort = query_db(query_sort, &application_table_namespace, &connection).unwrap();
        let result_sort_and_where =
            query_db(query_sort_and_where, &application_table_namespace, &connection).unwrap();

        assert!(
            result_sort.result
                == vec![vec!["d", "123", "\"asjd1j223\""], vec!["c", "53", "\"asjd1j223\""]]
        );
        assert!(result_sort_and_where.result == vec![vec!["d", "123", "\"asjd1j223\""]]);
        remove_db_if_exists(DB_PATH).unwrap();
    }

    #[test]
    fn test_offset_create_db_insert_query() {
        let insert_1 = DatabaseInsertEntry {
            table_id: 0,
            data: vec!["a".to_string(), "654".to_string(), "\"asjd1j223\"".to_string()],
        };
        let insert_2 = DatabaseInsertEntry {
            table_id: 0,
            data: vec!["b".to_string(), "3455".to_string(), "\"asjd1j223\"".to_string()],
        };
        let insert_3 = DatabaseInsertEntry {
            table_id: 0,
            data: vec!["c".to_string(), "53".to_string(), "\"asjd1j223\"".to_string()],
        };
        let insert_4 = DatabaseInsertEntry {
            table_id: 0,
            data: vec!["d".to_string(), "123".to_string(), "\"asjd1j223\"".to_string()],
        };

        let query_sort = DatabaseQuery {
            number_of_fields: 3,
            table_id: 0,
            sorting_operations: vec![SortingOperation { field_id: 0, descending: true }],
            where_operations: vec![],
            limit: 2,
            offset: 1,
        };
        let query_sort_and_where = DatabaseQuery {
            number_of_fields: 3,
            table_id: 0,
            sorting_operations: vec![SortingOperation { field_id: 0, descending: true }],
            where_operations: vec![WhereOperation { field_id: 1, value: "123".to_string() }],
            limit: 2,
            offset: 1,
        };

        let database_create_table: DatabaseCreateTable = DatabaseCreateTable {
            table_id: 0,
            fields: vec![SQLFieldTypes::Text, SQLFieldTypes::Text, SQLFieldTypes::Text],
        };

        const DB_PATH: &str = "./test2.db3";
        remove_db_if_exists(DB_PATH).unwrap();

        let connection = rusqlite::Connection::open(DB_PATH).unwrap();
        let application_table_namespace = "firstcontract".to_string();
        create_table(database_create_table, &application_table_namespace, &connection).unwrap();
        insert_entry(insert_1, &application_table_namespace, &connection).unwrap();
        insert_entry(insert_2, &application_table_namespace, &connection).unwrap();
        insert_entry(insert_3, &application_table_namespace, &connection).unwrap();
        insert_entry(insert_4, &application_table_namespace, &connection).unwrap();

        let result_sort = query_db(query_sort, &application_table_namespace, &connection).unwrap();
        let result_sort_and_where =
            query_db(query_sort_and_where, &application_table_namespace, &connection).unwrap();

        //println!("{result_sort:?}");
        //println!("{result_sort_and_where:?}");
        assert!(
            result_sort.result
                == vec![["c", "53", "\"asjd1j223\""], ["b", "3455", "\"asjd1j223\""]]
        );
        assert!(result_sort_and_where.result.len() == 0);
        remove_db_if_exists(DB_PATH).unwrap();
    }
    #[test]
    fn test_determistic_sorting_create_db_insert_query() {
        let insert_1 = DatabaseInsertEntry {
            table_id: 0,
            data: vec!["a".to_string(), "555".to_string(), "\"asjd1j223\"".to_string()],
        };
        let insert_2 = DatabaseInsertEntry {
            table_id: 0,
            data: vec!["b".to_string(), "555".to_string(), "\"asjd1j223\"".to_string()],
        };
        let insert_3 = DatabaseInsertEntry {
            table_id: 0,
            data: vec!["c".to_string(), "444".to_string(), "\"asjd1j223\"".to_string()],
        };
        let insert_4 = DatabaseInsertEntry {
            table_id: 0,
            data: vec!["d".to_string(), "444".to_string(), "\"asjd1j223\"".to_string()],
        };

        let query_sort = DatabaseQuery {
            number_of_fields: 3,
            table_id: 0,
            sorting_operations: vec![SortingOperation { field_id: 1, descending: true }],
            where_operations: vec![],
            limit: 10,
            offset: 0,
        };
        let query_sort_and_where = DatabaseQuery {
            number_of_fields: 3,
            table_id: 0,
            sorting_operations: vec![SortingOperation { field_id: 1, descending: false }],
            where_operations: vec![],
            limit: 10,
            offset: 0,
        };

        let database_create_table: DatabaseCreateTable = DatabaseCreateTable {
            table_id: 0,
            fields: vec![SQLFieldTypes::Text, SQLFieldTypes::Text, SQLFieldTypes::Text],
        };

        const DB_PATH: &str = "./test3.db3";
        remove_db_if_exists(DB_PATH).unwrap();

        let connection = rusqlite::Connection::open(DB_PATH).unwrap();
        let application_table_namespace = "firstcontract".to_string();
        create_table(database_create_table, &application_table_namespace, &connection).unwrap();
        insert_entry(insert_1, &application_table_namespace, &connection).unwrap();
        insert_entry(insert_2, &application_table_namespace, &connection).unwrap();
        insert_entry(insert_3, &application_table_namespace, &connection).unwrap();
        insert_entry(insert_4, &application_table_namespace, &connection).unwrap();

        let result_sort = query_db(query_sort, &application_table_namespace, &connection).unwrap();
        let result_sort_and_where =
            query_db(query_sort_and_where, &application_table_namespace, &connection).unwrap();

        assert!(
            result_sort.result
                == vec![
                    ["b", "555", "\"asjd1j223\""],
                    ["a", "555", "\"asjd1j223\""],
                    ["d", "444", "\"asjd1j223\""],
                    ["c", "444", "\"asjd1j223\""]
                ]
        );
        assert!(
            result_sort_and_where.result
                == vec![
                    ["d", "444", "\"asjd1j223\""],
                    ["c", "444", "\"asjd1j223\""],
                    ["b", "555", "\"asjd1j223\""],
                    ["a", "555", "\"asjd1j223\""]
                ]
        );
        remove_db_if_exists(DB_PATH).unwrap();
    }
    #[test]
    fn test_sql_sort_string_numbers_does_not_work() {
        let insert_1 = DatabaseInsertEntry {
            table_id: 0,
            data: vec!["a".to_string(), "93883484".to_string(), "\"asjd1j223\"".to_string()],
        };
        let insert_2 = DatabaseInsertEntry {
            table_id: 0,
            data: vec!["b".to_string(), "54823124".to_string(), "\"asjd1j223\"".to_string()],
        };
        let insert_3 = DatabaseInsertEntry {
            table_id: 0,
            data: vec!["c".to_string(), "22222222".to_string(), "\"asjd1j223\"".to_string()],
        };
        let insert_4 = DatabaseInsertEntry {
            table_id: 0,
            data: vec!["d".to_string(), "3405".to_string(), "\"asjd1j223\"".to_string()],
        };

        let query_sort = DatabaseQuery {
            number_of_fields: 3,
            table_id: 0,
            sorting_operations: vec![SortingOperation { field_id: 1, descending: false }],
            where_operations: vec![],
            limit: 10,
            offset: 0,
        };

        let database_create_table: DatabaseCreateTable = DatabaseCreateTable {
            table_id: 0,
            fields: vec![SQLFieldTypes::Text, SQLFieldTypes::Text, SQLFieldTypes::Text],
        };

        const DB_PATH: &str = "./test4.db3";
        remove_db_if_exists(DB_PATH).unwrap();

        let connection = rusqlite::Connection::open(DB_PATH).unwrap();
        let application_table_namespace = "firstcontract".to_string();
        create_table(database_create_table, &application_table_namespace, &connection).unwrap();
        insert_entry(insert_1, &application_table_namespace, &connection).unwrap();
        insert_entry(insert_2, &application_table_namespace, &connection).unwrap();
        insert_entry(insert_3, &application_table_namespace, &connection).unwrap();
        insert_entry(insert_4, &application_table_namespace, &connection).unwrap();

        let result_sort = query_db(query_sort, &application_table_namespace, &connection).unwrap();

        //println!("{result_sort:?}");
        assert!(
            result_sort.result
                == vec![
                    ["c", "22222222", "\"asjd1j223\""],
                    ["d", "3405", "\"asjd1j223\""],
                    ["b", "54823124", "\"asjd1j223\""],
                    ["a", "93883484", "\"asjd1j223\""]
                ]
        );
        remove_db_if_exists(DB_PATH).unwrap();
    }

    #[test]
    fn test_sql_update() {
        let insert_1 = DatabaseInsertEntry {
            table_id: 0,
            data: vec!["a".to_string(), "93883484".to_string(), "\"asjd1j223\"".to_string()],
        };
        let insert_2 = DatabaseInsertEntry {
            table_id: 0,
            data: vec!["b".to_string(), "54823124".to_string(), "\"asjd1j223\"".to_string()],
        };
        let insert_3 = DatabaseInsertEntry {
            table_id: 0,
            data: vec!["c".to_string(), "22222222".to_string(), "\"asjd1j223\"".to_string()],
        };
        let insert_4 = DatabaseInsertEntry {
            table_id: 0,
            data: vec!["d".to_string(), "3405".to_string(), "\"asjd1j223\"".to_string()],
        };

        let query_where_d = DatabaseQuery {
            number_of_fields: 3,
            table_id: 0,
            sorting_operations: vec![],
            where_operations: vec![WhereOperation { field_id: 0, value: "d".to_string() }],
            limit: 10,
            offset: 0,
        };

        let database_create_table: DatabaseCreateTable = DatabaseCreateTable {
            table_id: 0,
            fields: vec![SQLFieldTypes::Text, SQLFieldTypes::Text, SQLFieldTypes::Text],
        };

        const DB_PATH: &str = "./test5.db3";
        remove_db_if_exists(DB_PATH).unwrap();

        let connection = rusqlite::Connection::open(DB_PATH).unwrap();
        let application_table_namespace = "firstcontract".to_string();
        create_table(database_create_table, &application_table_namespace, &connection).unwrap();
        insert_entry(insert_1, &application_table_namespace, &connection).unwrap();
        insert_entry(insert_2, &application_table_namespace, &connection).unwrap();
        insert_entry(insert_3, &application_table_namespace, &connection).unwrap();
        insert_entry(insert_4, &application_table_namespace, &connection).unwrap();

        let update = DatabaseUpdateEntry {
            table_id: 0,
            select_on_primary_key_value: "d".to_string(),
            data: vec!["updated_d_val".to_string(), "\"updated_d_val_2\"".to_string()],
        };
        update_entry(update, &application_table_namespace, &connection).unwrap();
        let result_where_d =
            query_db(query_where_d, &application_table_namespace, &connection).unwrap();

        assert!(result_where_d.result == vec![["d", "updated_d_val", "\"updated_d_val_2\""]]);
        remove_db_if_exists(DB_PATH).unwrap();
    }
}

use super::host_communication::DatabaseCreateTable;
use super::host_communication::DatabaseInsertEntry;
use super::host_communication::DatabaseQuery;
use super::host_communication::DatabaseQueryResult;
use super::host_communication::DatabaseUpdateEntry;
use crate::application::sql::host_communication::parse_query_json;
use crate::application::sql::query_builder::Query;
use rusqlite::Result;
