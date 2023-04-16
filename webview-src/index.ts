import { invoke } from '@tauri-apps/api/tauri'

/**
 * Open database options
 */
export interface OpenOptions {
  /** Disable foreign keys validation */
  disable_foreign_keys?: boolean
}

/**
 * List of queries to run in batch
 */
export type BatchQueries = [string, unknown[]][];

/**
 * API Tauri 
 */
export class Taurusqlite {
  /**
   * Database path
   */
  private path: string = '';

  /**
   * Constructor
   * 
   * @param path - Database path
   */
  private constructor(path: string) {
    this.path = path;
  }

  /**
   * Connect to the database
   * 
   * @param options - Opening options
   * 
   * @returns True on success
   */
  private connect(options?: OpenOptions): Promise<boolean> {
    if (options === undefined) {
      options = {};
    }
    return invoke('plugin:taurusqlite|open', {dbPath: this.path, options});
  }
  
  /**
   * Open store
   * 
   * @param options  - Opening options
   * 
   * @returns Api object
   * 
   * @example
   * 
   * ```ts
   * const db = await Taurusqlite.load();
   * ```
   */
  public static load(options?: OpenOptions): Promise<Taurusqlite> {
    return new Promise<Taurusqlite>((resolve, reject) => {
      let instance = new Taurusqlite('');
      if (options === undefined) {
        options = {};
      }
      invoke('plugin:taurusqlite|load', {options}).then(storePath => {
        instance.path = storePath as string;
        resolve(instance);
      }).catch(reject);
    });
  }

  /**
   * Open database file
   * 
   * @param path - Path to the database file 
   * @param options  - Opening options
   * 
   * @returns Api object
   * 
   * @example
   * 
   * ```ts
   * const db = await Taurusqlite.open('/path/to/the/db');
   * ```
   */
  public static open(path: string, options?: OpenOptions): Promise<Taurusqlite> {
    return new Promise<Taurusqlite>((resolve, reject) => {
      let instance = new Taurusqlite(path);
      instance.connect(options).then(success => {
        if (success) {
          resolve(instance);
        } else {
          reject(`Unable to open database ${path}.`);
        }
      }).catch(reject);      
    });
  }

  /**
   * Set pragma value
   * 
   * @param key - Pragma key
   * @param value - Value to set
   * 
   * @returns True on success
   * 
   * @example
   * 
   * ```ts
   * const db = await Taurusqlite.open('/path/to/the/db');
   * db.setPragma('foreign_keys', 0);
   * ```
   */
  public setPragma(key: string, value: any): Promise<boolean> {
    return invoke('plugin:taurusqlite|set_pragma', {dbPath: this.path, key, value});
  }

  /**
   * Select in database
   * 
   * @param query - Select query
   * @param params - Query params
   * 
   * @returns Array with selected rows
   * 
   * @example
   * 
   * ```ts
   * const db = await Taurusqlite.open('/path/to/the/db');
   * const rows = await db.select<{id: number, name: string}>('SELECT id, name FROM person WHERE age >= ?1 and sex = ?2', [18, 'M']);
   * for (r of rows) {
   *   console.log(`${person.name} has id ${person.id}`);
   * }
   * ```
   */
  public select<T>(query: string, params?: unknown[]): Promise<Array<T>> {
    if (params === undefined) {
      params = [];
    }
    return invoke('plugin:taurusqlite|select', {dbPath: this.path, query, params});
  }

  /**
   * Select first row in selected rows
   * 
   * @param query - Select query
   * @param params - Query params
   * 
   * @returns Array with selected rows
   * 
   * @example
   * 
   * ```ts
   * const db = await Taurusqlite.open('/path/to/the/db');
   * db.selectFirst<{id: number, name: string}>('SELECT id, name FROM person WHERE age >= ?1 and sex = ?2', [18, 'M']).then(perso => {
   *   console.log(`${person.name} has id ${person.id}`);
   * }).catch((e) => {
   *   console.error('No adults');
   * });
   * ```
   */  
  public selectFirst<T>(query: string, params?: unknown[]): Promise<T> {
    return new Promise<T>((resolve, reject) => {
      invoke('plugin:taurusqlite|select', {dbPath: this.path, query, params}).then((results) => {
        if ((results as Array<T>).length > 0) {
          resolve((results as Array<T>)[0]);
        } else {
          reject(new Error('No results'));
        }
      }).catch(reject);
    });
  }

  /**
   * Execute a query
   * 
   * @param query - SQL query
   * @param params - Query params
   * 
   * @returns True on success
   * 
   * @example
   * 
   * ```ts
   * const db = await Taurusqlite.open('/path/to/the/db');
   * db.execute('DELETE FROM person WHERE age < ?1 and sex = ?2', [18, 'F']);
   * ```
   */
  public execute(query: string, params?: unknown[]): Promise<boolean> {
    if (params === undefined) {
      params = [];
    }
    return invoke('plugin:taurusqlite|execute', {dbPath: this.path, query, params});
  }

  /**
   * Execute a list of queries with on transaction (Rollback on error)
   * 
   * @param queries - List of queries
   * 
   * @returns True on success
   * 
   * @example
   * 
   * ```ts
   * const db = await Taurusqlite.open('/path/to/the/db');
   * db.batch([
   *  ['DELETE FROM person WHERE age < ?1', [18]],
   *  ['INSERT INTO person VALUES (NULL, ?1, ?2, ?3)', ['John', 16, 'F']]
   * ]);
   * ```
   */
  public batch(queries: BatchQueries): Promise<boolean> {
    return invoke('plugin:taurusqlite|batch', {dbPath: this.path, queries});
  }
}
