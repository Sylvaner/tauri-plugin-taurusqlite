/**
 * Open database options
 */
export interface OpenOptions {
    /** Disable foreign keys validation */
    disable_foreign_keys?: boolean;
}
/**
 * List of queries to run in batch
 */
export declare type BatchQueries = [string, unknown[]][];
/**
 * API Tauri
 */
export declare class Taurusqlite {
    /**
     * Database path
     */
    private path;
    /**
     * Constructor
     *
     * @param path - Database path
     */
    private constructor();
    /**
     * Connect to the database
     *
     * @param options - Opening options
     *
     * @returns True on success
     */
    private connect;
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
    static load(options?: OpenOptions): Promise<Taurusqlite>;
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
    static open(path: string, options?: OpenOptions): Promise<Taurusqlite>;
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
    setPragma(key: string, value: any): Promise<boolean>;
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
    select<T>(query: string, params?: unknown[]): Promise<Array<T>>;
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
    selectFirst<T>(query: string, params?: unknown[]): Promise<T>;
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
    execute(query: string, params?: unknown[]): Promise<boolean>;
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
    batch(queries: BatchQueries): Promise<boolean>;
}
