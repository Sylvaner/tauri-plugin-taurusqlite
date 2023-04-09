export declare class Taurusqlite {
    private path;
    private constructor();
    private connect;
    static open(databasePath: string): Promise<Taurusqlite>;
}
