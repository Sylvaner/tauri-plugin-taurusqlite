import { invoke } from '@tauri-apps/api/tauri'

export class Taurusqlite {
  private path: string = '';

  private constructor(path: string) {
    this.path = path;
  }

  private connect(): Promise<boolean> {
    return invoke('plugin:taurusqlite|open', {database_path: this.path});
  }
  
  public static open(databasePath: string): Promise<Taurusqlite> {
    return new Promise<Taurusqlite>((resolve, reject) => {
      let instance = new Taurusqlite(databasePath);
      instance.connect().then(success => {
        if (success) {
          resolve(instance);
        } else {
          reject(`Unable to open database ${databasePath}.`);
        }
      }).catch(reject);      
    });
  }
}
