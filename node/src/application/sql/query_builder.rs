pub struct Query {
    pub iterator: u32,
    pub query: String,
    pub parameters: Vec<String>,
}
impl Query {
    fn calculate_table_name(table_id: u32, application_table_namespace: &str) -> String {
        let table_name = format!("site{}{}", application_table_namespace, table_id);
        return table_name;
    }
    pub fn select(table_id: u32, application_table_namespace: &str) -> Query {
        let iterator: u32 = 1;
        let table_name = Query::calculate_table_name(table_id, application_table_namespace);
        Query {
            iterator: iterator,
            query: format!("SELECT * FROM {}\n", table_name),
            parameters: vec![],
        }
    }

    //sql where, if no select operations passed, does not add  "WHERE§" to sql query
    pub fn sql_where(mut self, where_operations: Vec<WhereOperation>) -> Query {
        /*
         WHERE
           "id" = $1
           AND "id" = $2
        */
        if where_operations.len() > 0 {
            let mut command: String = String::from("WHERE\n");
            let mut first = true;
            for where_operation in where_operations {
                if first {
                    command += &format!("field{} = ?{}\n", where_operation.field_id, self.iterator);
                    first = false;
                } else {
                    command +=
                        &format!("AND field{} = ?{}\n", where_operation.field_id, self.iterator);
                }

                let param_value = where_operation.value;

                self.parameters.push(param_value);
                self.iterator += 1;
            }
            self.query += &command;
        }
        return self;
    }
    pub fn sql_sort_determistic(mut self, sorting_operations: Vec<SortingOperation>) -> Query {
        //ORDER BY Country ASC, CustomerName DESC;
        let mut command: String = String::from("ORDER BY\n  ");
        let mut first = true;
        for sorting_operation in sorting_operations {
            let direction: String;
            if sorting_operation.descending {
                direction = String::from("DESC");
            } else {
                direction = String::from("ASC");
            }
            if first {
                command += &format!("field{} {}", sorting_operation.field_id, direction);
                first = false;
            } else {
                command += &format!("\n, field{} {}", sorting_operation.field_id, direction);
            }
        }

        //ensure deterministic queries by always sorting all queries on unique primary key values
        if first {
            command += "__PRIMKEY DESC";
        } else {
            command += "\n, __PRIMKEY DESC";
        }

        self.query += &command;
        return self;
    }
    pub fn sql_limit(mut self, limit: u32, offset: u32) -> Query {
        if limit == 0 {
            return self;
        }

        let template = format!("\nLIMIT ?{} OFFSET ?{}", self.iterator, self.iterator + 1);
        self.iterator += 1;
        self.iterator += 1;

        self.query += &template;
        self.parameters.append(&mut vec![limit.to_string(), offset.to_string()]);
        return self;
    }

    pub fn calculate(self) -> (String, Vec<String>) {
        return (self.query, self.parameters);
    }
    pub fn create_table(
        database_create_table: DatabaseCreateTable,
        application_table_namespace: &str,
    ) -> (String, Vec<String>) {
        let mut field_name = 0;
        let mut list_of_fields = vec![];

        //always integer primary key __PRIMKEY
        list_of_fields.push("__PRIMKEY INTEGER PRIMARY KEY NOT NULL".to_string());

        for field in database_create_table.fields {
            let field_type = match field {
                SQLFieldTypes::Integer => "INTEGER",
                SQLFieldTypes::Text => "TEXT",
                SQLFieldTypes::Blob => "BLOB",
            };

            let message = format!("field{} {} NOT NULL", field_name, field_type);
            list_of_fields.push(message);

            field_name += 1;
        }
        let fields = list_of_fields.join(", ");
        let table_name = Query::calculate_table_name(
            database_create_table.table_id,
            application_table_namespace,
        );

        //because cannot parameterize dynamic sql table names must limit names to numbers and rely on fact numbers cannot be sql injected
        let sql_command = format!("CREATE TABLE {} ({})", table_name, fields);

        (sql_command, vec![])
    }

    pub fn sql_insert(
        database_insert_entry: DatabaseInsertEntry,
        application_table_namespace: &str,
    ) -> (String, Vec<String>) {
        //string entry needs value to be "test" for numbe 123 so json is ""string"" "12312"
        /*INSERT INTO person (name, data) VALUES (?1, ?2) */
        let table_name = Query::calculate_table_name(
            database_insert_entry.table_id,
            application_table_namespace,
        );

        let command = format!("INSERT INTO {}", table_name);
        let mut var_id = 1;

        let mut field_string = "(".to_string();
        let mut parameter_string = "(".to_string();

        let data_len = database_insert_entry.data.len();

        for i in 0..data_len {
            //last
            if i == data_len - 1 {
                field_string += &format!("field{}", i);
                parameter_string += &format!("?{var_id}");
            } else {
                field_string += &format!("field{}, ", i);
                parameter_string += &format!("?{var_id}, ");
            }
            var_id += 1;
        }
        field_string += ")";
        parameter_string += ")";

        let sql_command = format!("{command} {field_string} VALUES {parameter_string}");
        return (sql_command, database_insert_entry.data);
    }

    pub fn sql_update(
        database_update_entry: DatabaseUpdateEntry,
        application_table_namespace: &str,
    ) -> (String, Vec<String>) {
        /*
        UPDATE Customers
        SET ContactName = 'Alfred Schmidt', City= 'Frankfurt'
        WHERE CustomerID = 1;
        */
        let table_name = Query::calculate_table_name(
            database_update_entry.table_id,
            application_table_namespace,
        );
        let command = format!("UPDATE {}", table_name);
        let mut var_id = 1;

        let mut set_params_string = "".to_string();

        let data_len = database_update_entry.data.len();

        //0 is primary key, cannot update primary key, so start all fields from 1
        for i in 1..(data_len + 1) {
            let last_data = i == (data_len + 1) - 1;
            if last_data {
                set_params_string += &format!("field{i} = ?{var_id}");
            } else {
                set_params_string += &format!("field{i} = ?{var_id}, ");
            }
            var_id += 1;
        }

        let sql_command =
            format!("{command} \n SET {set_params_string} \n WHERE field0 = ?{var_id}");
        //#[warn(unused_assignments)]
        //var_id += 1;
        let mut params = database_update_entry.data.clone();
        params.push(database_update_entry.select_on_primary_key_value);
        return (sql_command, params);
    }
}

use crate::application::sql::host_communication::{
    DatabaseCreateTable, DatabaseInsertEntry, DatabaseUpdateEntry, SQLFieldTypes, SortingOperation,
    WhereOperation,
};

#[cfg(test)]
mod tests {
    use crate::application::sql::{
        host_communication::{
            DatabaseCreateTable, DatabaseInsertEntry, DatabaseQuery, DatabaseUpdateEntry,
            SQLFieldTypes, SortingOperation, WhereOperation,
        },
        query_builder::Query,
    };

    #[test]
    fn test_build_create_table() {
        let database_create_table: DatabaseCreateTable = DatabaseCreateTable {
            table_id: 0,
            fields: vec![SQLFieldTypes::Integer, SQLFieldTypes::Text, SQLFieldTypes::Blob],
        };
        let (template, params) = super::Query::create_table(database_create_table, "contract1_");
        let template_shouldbe = "CREATE TABLE sitecontract1_0 (__PRIMKEY INTEGER PRIMARY KEY NOT NULL, field0 INTEGER NOT NULL, field1 TEXT NOT NULL, field2 BLOB NOT NULL)";

        println!("{:?}", template);
        println!("{:?}", template_shouldbe);
        assert!(params.len() == 0);
        assert!(template == template_shouldbe);
    }

    #[test]
    fn test_build_query() {
        let query = DatabaseQuery {
            number_of_fields: 3,
            table_id: 0,
            sorting_operations: vec![SortingOperation { field_id: 2, descending: true }],
            where_operations: vec![WhereOperation { field_id: 0, value: String::from("33") }],
            limit: 123,
            offset: 5,
        };

        let (template, params) = Query::select(query.table_id, "contract1_")
            .sql_where(query.where_operations)
            .sql_sort_determistic(query.sorting_operations)
            .sql_limit(query.limit, query.offset)
            .calculate();

        let template_shouldbe = "SELECT * FROM sitecontract1_0
WHERE
field0 = ?1
ORDER BY
  field2 DESC
, __PRIMKEY DESC
LIMIT ?2 OFFSET ?3";
        //println!("{template}");
        //println!("{:?}", params);
        assert!(template == template_shouldbe);
        assert!(params == vec!["33", "123", "5"]);
    }

    #[test]
    fn test_insert_entry() {
        let insert_entry = DatabaseInsertEntry {
            table_id: 0,
            data: vec![
                "\"STRING_EXAMPLE\"".to_string(),
                "123123123".to_string(),
                "\"asjd1j223\"".to_string(),
            ],
        };
        let (template, params) = Query::sql_insert(insert_entry, "contract1_");
        //println!("{template}");
        //println!("{:?}", params);
        assert!(
            template == "INSERT INTO sitecontract1_0 (field0, field1, field2) VALUES (?1, ?2, ?3)"
        );
        assert!(params == vec!["\"STRING_EXAMPLE\"", "123123123", "\"asjd1j223\""]);
    }
    #[test]
    fn test_update_entry() {
        let update_entry = DatabaseUpdateEntry {
            table_id: 0,
            select_on_primary_key_value: "KEYEXAMPLE".to_string(),
            data: vec![
                "\"STRING_EXAMPLE\"".to_string(),
                "123123123".to_string(),
                "\"asjd1j223\"".to_string(),
            ],
        };
        let (template, params) = Query::sql_update(update_entry, "contract1_");
        //println!("{template}");
        //println!("{:?}", params);
        assert!(
            template
                == "UPDATE sitecontract1_0 
 SET field1 = ?1, field2 = ?2, field3 = ?3 
 WHERE field0 = ?4"
        );
        assert!(params == vec!["\"STRING_EXAMPLE\"", "123123123", "\"asjd1j223\"", "KEYEXAMPLE"]);
    }
}
