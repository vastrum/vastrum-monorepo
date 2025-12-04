declare module 'runtimebindings' {
  /**
   * Metadata
   */
  export function blocktime(): bigint;
  /**
   * Register pages
   */
  export function registerstaticroute(route: string, html: string): void;
  /**
   * register_route: func(route: string, data: string, templatePath: string);
   * Database operations
   */
  export function dbquery(json: string): string;
  export function dbcreatetable(json: string): void;
  export function dbupdateentry(json: string): void;
  export function dbinsertentry(json: string): void;
}
