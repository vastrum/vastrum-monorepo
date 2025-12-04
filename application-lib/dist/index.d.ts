//#region src/typeReflector.d.ts
declare enum DataType {
  Bool = 0,
  Uint64 = 1,
  String = 2,
}
interface ColumnMetadata {
  propertyName: string;
  type: DataType;
}
interface RepositoryMetadata {
  target: Function;
  tableID: u64;
  columns: ColumnMetadata[];
  columnToIDMapping: Map<String, u64>;
  currentColumnID: u64;
  amountOfColumns: u64;
}
declare function Column(type: DataType, array?: boolean, ptype?: any | DataType): (target: any, propertyName: string) => void;
//#endregion
//#region src/dblib.d.ts
type WhereOperator = "==";
type OrderDirection = "asc" | "desc";
declare class TypedQuery<T> {
  private repositoryMetaData;
  private entityClass;
  private whereOperations;
  private sortingOperations;
  private limitCount;
  private offsetCount;
  constructor(repositoryMetaData: RepositoryMetadata, entityClass: new () => T);
  where<K$1 extends keyof T>(field: K$1, operator: WhereOperator, value: string | number): TypedQuery<T>;
  orderBy<K$1 extends keyof T>(field: K$1, direction?: OrderDirection): TypedQuery<T>;
  limit(count: number): TypedQuery<T>;
  offset(count: number): TypedQuery<T>;
  get(): T[];
  private clone;
}
type FieldBindings<T> = { readonly [K in keyof T]: K };
declare class Repository<T> {
  private entityClass;
  private tableID;
  private metadata;
  columns: FieldBindings<T>;
  constructor(entityClass: new () => T, tableID: u64);
  query(): TypedQuery<T>;
  insert(value: T): void;
  update(select_on_primary_key_value: u64, value: T): void;
  createTable(): void;
}
//#endregion
//#region src/index.d.ts
type u64 = number;
type i64 = number;
type u32 = number;
type i32 = number;
declare function register_static_route(route: string, html: string): void;
declare function register_route(route: string, data: string, template_path: string): void;
declare function block_time(): number;
type DatabaseQuery = {
  table_id: u32;
  number_of_fields: u32;
  sorting_operations: SortingOperation[];
  where_operations: WhereOperation[];
  limit: u32;
  offset: u32;
};
type DatabaseQueryResult = {
  result: string[][];
};
type SortingOperation = {
  field_id: u32;
  descending: boolean;
};
type WhereOperation = {
  field_id: u32;
  value: string;
};
type DatabaseCreateTable = {
  table_id: u32;
  fields: SQLFieldTypes[];
};
declare enum SQLFieldTypes {
  Integer = 0,
  Text = 1,
  Blob = 2,
}
type DatabaseInsertEntry = {
  table_id: u32;
  data: string[];
};
type DatabaseUpdateEntry = {
  table_id: u32;
  select_on_primary_key_value: string;
  data: string[];
};
declare function db_insert_entry(databaseInsertEntry: DatabaseInsertEntry): void;
declare function db_update_entry(databaseUpdateEntry: DatabaseUpdateEntry): void;
declare function db_query(databaseQuery: DatabaseQuery): string[][];
declare function db_create_table(databaseCreateTable: DatabaseCreateTable): void;
//#endregion
export { Column, DataType, DatabaseCreateTable, DatabaseInsertEntry, DatabaseQuery, DatabaseQueryResult, DatabaseUpdateEntry, Repository, SQLFieldTypes, SortingOperation, TypedQuery, WhereOperation, block_time, db_create_table, db_insert_entry, db_query, db_update_entry, i32, i64, register_route, register_static_route, u32, u64 };