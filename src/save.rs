use std::{error::Error, fs::File};

use serde::de::DeserializeOwned;
use sqlite::Connection;

use crate::bwarm::{
    interface::BwarmEntry,
    types::{Party, Share, Work},
};

fn save_object<T: BwarmEntry + DeserializeOwned>(conn: &Connection) -> Result<(), Box<dyn Error>> {
    let fullpath = dirs::download_dir().unwrap().join(T::filename());

    let file = File::open(fullpath)?;
    let mut rdr = csv::ReaderBuilder::new()
        .delimiter(b'\t')
        .has_headers(true)
        .from_reader(file);

    let headers = rdr.headers()?.clone();
    dbg!(&headers);
    let mut stmt = T::stmt(conn)?;
    for entry in rdr.records() {
        let x = entry?;
        let obj: T = x.deserialize(Some(&headers))?;
        obj.insert(&mut stmt)?;
        stmt.reset()?;
    }
    Ok(())
}

fn wrap_tx<T>(
    conn: &Connection,
    f: fn(&Connection) -> Result<T, Box<dyn Error>>,
) -> Result<T, Box<dyn Error>> {
    conn.execute("BEGIN TRANSACTION")
        .expect("failed to start tx");
    match f(conn) {
        Ok(x) => {
            conn.execute("COMMIT")?;
            Ok(x)
        }
        Err(e) => {
            conn.execute("ROLLBACK")?;
            Err(e)
        }
    }
}

fn migrate(conn: &Connection) -> Result<(), sqlite::Error> {
    Party::migrate(conn)?;
    Work::migrate(conn)?;
    Share::migrate(conn)?;
    // conn.execute("CREATE INDEX idx_party ON parties(id)")
    Ok(())
}

pub fn migrate_and_save(db_path: &str) {
    let conn = sqlite::open(db_path).expect("failed to create db");
    migrate(&conn).unwrap();

    wrap_tx(&conn, save_object::<Party>).unwrap();
    wrap_tx(&conn, save_object::<Work>).unwrap();
    wrap_tx(&conn, save_object::<Share>).unwrap();
}
