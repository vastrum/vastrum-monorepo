/// <reference path="./generated/types/guest/import/hostbindings.d.ts" />
import {
  blocktime,
  registerstaticroute,
  dbquery,
  dbcreatetable,
  dbinsertentry,
  dbupdateentry,
} from "runtimebindings";

/*
for running vitests
function dbinsertentry() {}
function dbquery() {}
function dbcreatetable() {}
function dbupdateentry() {}
*/

export type u64 = number;
export type i64 = number;
export type u32 = number;
export type i32 = number;

export function register_static_route(route: string, html: string) {
  registerstaticroute(route, html);
}
export function register_route(
  route: string,
  data: string,
  template_path: string
) { }
export function block_time(): number {
  return Number(blocktime());
}
export type DatabaseQuery = {
  table_id: u32;
  number_of_fields: u32;
  sorting_operations: SortingOperation[];
  where_operations: WhereOperation[];
  limit: u32;
  offset: u32;
};

export type DatabaseQueryResult = {
  result: string[][];
};

export type SortingOperation = {
  field_id: u32;
  descending: boolean; //descending or ascending direction in this sort
};

export type WhereOperation = {
  field_id: u32;
  value: string;
};

export type DatabaseCreateTable = {
  table_id: u32;
  fields: SQLFieldTypes[];
};

export enum SQLFieldTypes {
  Integer,
  Text,
  Blob,
}

export type DatabaseInsertEntry = {
  table_id: u32;
  data: string[];
};

export type DatabaseUpdateEntry = {
  table_id: u32;
  select_on_primary_key_value: string;
  data: string[];
};
export function db_insert_entry(databaseInsertEntry: DatabaseInsertEntry) {
  dbinsertentry(JSON.stringify(databaseInsertEntry));
}
export function db_update_entry(databaseUpdateEntry: DatabaseUpdateEntry) {
  dbupdateentry(JSON.stringify(databaseUpdateEntry));
}

export function db_query(databaseQuery: DatabaseQuery): string[][] {
  let jsonresponse = dbquery(JSON.stringify(databaseQuery));
  let parsed: DatabaseQueryResult = JSON.parse(jsonresponse);
  return parsed.result;
}
export function db_create_table(databaseCreateTable: DatabaseCreateTable) {
  dbcreatetable(JSON.stringify(databaseCreateTable));
}

export * from "./dblib";
