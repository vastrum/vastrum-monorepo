import { Deserialize, encode } from "./serializer";
import {
  Column,
  DataType,
  metadataStorage,
  TypeReflector,
  type RepositoryMetadata,
} from "./typeReflector";

import {
  type u64,
  block_time,
  register_static_route,
  register_route,
  db_insert_entry,
  db_create_table,
  db_query,
  type DatabaseCreateTable,
  type DatabaseInsertEntry,
  db_update_entry,
  type DatabaseQuery,
  type SortingOperation,
  type WhereOperation,
  SQLFieldTypes,
  type DatabaseUpdateEntry,
} from "./index";

//TODO MORE OPERANDS
type WhereOperator = "=="; // | "!=" | "<" | "<=" | ">" | ">=";

// Order by types
type OrderDirection = "asc" | "desc";

// Query builder
export class TypedQuery<T> {
  private whereOperations: WhereOperation[] = [];
  private sortingOperations: SortingOperation[] = [];
  private limitCount: number = 0;
  private offsetCount: number = 0;
  constructor(
    private repositoryMetaData: RepositoryMetadata,
    private entityClass: new () => T
  ) { }

  // Where methods
  where<K extends keyof T>(
    field: K,
    operator: WhereOperator,
    value: string | number
  ): TypedQuery<T> {
    const newQuery = this.clone();
    let columnID = this.repositoryMetaData.columnToIDMapping.get(
      field.toString()
    );
    if (columnID != undefined) {
      newQuery.whereOperations.push({ field_id: columnID, value: value.toString() });
    }
    return newQuery;
  }

  orderBy<K extends keyof T>(
    field: K,
    direction: OrderDirection = "asc"
  ): TypedQuery<T> {
    const newQuery = this.clone();
    let columnID = this.repositoryMetaData.columnToIDMapping.get(
      field.toString()
    );
    if (columnID != undefined) {
      newQuery.sortingOperations.push({
        field_id: columnID,
        descending: direction == "desc",
      });
    }
    return newQuery;
  }

  limit(count: number): TypedQuery<T> {
    const newQuery = this.clone();
    newQuery.limitCount = count;
    return newQuery;
  }

  offset(count: number): TypedQuery<T> {
    const newQuery = this.clone();
    newQuery.offsetCount = count;
    return newQuery;
  }

  get(): T[] {
    let databaseQuery: DatabaseQuery = {
      table_id: this.repositoryMetaData.tableID,
      number_of_fields: this.repositoryMetaData.amountOfColumns,
      sorting_operations: this.sortingOperations,
      where_operations: this.whereOperations,
      limit: this.limitCount,
      offset: this.offsetCount,
    };
    let result = db_query(databaseQuery);

    let parsed_results: T[] = [];
    for (let i = 0; i < result.length; i++) {
      let row = result[i];
      parsed_results.push(Deserialize(row, this.entityClass));
    }
    return parsed_results;
  }

  private clone(): TypedQuery<T> {
    const newQuery = new TypedQuery<T>(
      this.repositoryMetaData,
      this.entityClass
    );
    newQuery.whereOperations = this.whereOperations;
    newQuery.sortingOperations = this.sortingOperations;
    newQuery.limitCount = this.limitCount;
    newQuery.offsetCount = this.offsetCount;
    return newQuery;
  }
}

type FieldBindings<T> = {
  readonly [K in keyof T]: K;
};

function createFieldBindings<T>(): FieldBindings<T> {
  return new Proxy({} as FieldBindings<T>, {
    get(target, prop) {
      return prop;
    },
  });
}
export class Repository<T> {
  private metadata: RepositoryMetadata;
  public columns: FieldBindings<T>;
  constructor(private entityClass: new () => T, private tableID: u64) {
    let req = metadataStorage.getEntityMetadata(this.entityClass);
    this.metadata = req;
    this.columns = createFieldBindings<T>();
  }

  query(): TypedQuery<T> {
    return new TypedQuery<T>(this.metadata, this.entityClass);
  }

  insert(value: T) {
    let data: string[] = [];
    for (let i = 0; i < this.metadata.columns.length; i++) {
      let column = this.metadata.columns[i];
      data.push(encode(value[column.propertyName], column.type));
    }

    let databaseInsertEntry: DatabaseInsertEntry = {
      data: data,
      table_id: this.metadata.tableID,
    };
    db_insert_entry(databaseInsertEntry);
  }
  update(select_on_primary_key_value: u64, value: T) {
    let data: string[] = [];

    //skip first number which is id which cannot be skipped
    for (let i = 1; i < this.metadata.columns.length; i++) {
      let column = this.metadata.columns[i];
      data.push(encode(value[column.propertyName], column.type));
    }

    let updateParentPost: DatabaseUpdateEntry = {
      select_on_primary_key_value: select_on_primary_key_value.toString(),
      data: data,
      table_id: this.metadata.tableID,
    };
    db_update_entry(updateParentPost);
  }
  createTable() {
    let tableFields: SQLFieldTypes[] = [];

    for (let i = 0; i < this.metadata.columns.length; i++) {
      let column = this.metadata.columns[i];

      tableFields.push(SQLFieldTypes.Text);
    }

    let postRepliesTable: DatabaseCreateTable = {
      table_id: this.tableID,
      fields: tableFields,
    };
    db_create_table(postRepliesTable);
  }
}

function getDB<T>(entityClass: new () => T, tableID: u64): Repository<T> {
  return new Repository(entityClass, tableID);
}

export { Column, DataType } from "./typeReflector";
