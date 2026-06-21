use std::{error::Error, fs::File, path::Path};

use libsql::{Connection, Transaction};
use serde::de::DeserializeOwned;

use crate::bwarm::{
    interface::BwarmEntry,
    types::{Party, Release, Share, Work},
};

pub async fn save_object<T: BwarmEntry + DeserializeOwned>(
    tx: &Transaction,
    bwarm_dir: &Path,
) -> Result<(), Box<dyn Error>> {
    let fullpath = bwarm_dir.join(T::filename());

    let file = File::open(fullpath)?;
    let mut rdr = csv::ReaderBuilder::new()
        .delimiter(b'\t')
        .flexible(true)
        .has_headers(true)
        .from_reader(file);
    let headers = rdr.headers()?.clone();

    let mut stmt = T::prepare(tx).await?;
    let mut sum = 0;
    for entry in rdr.records() {
        let mut x = entry?;
        while x.len() < headers.len() {
            x.push_field("");
        }

        let obj = match x.deserialize::<T>(Some(&headers)) {
            Ok(x) => x,
            Err(e) => {
                eprintln!("{e}");
                continue;
            }
        };
        obj.insert(&mut stmt).await?;
        stmt.reset();
        sum += 1;
    }
    println!("INSERTED {}", sum);
    Ok(())
}

pub async fn migrate(conn: &Connection) -> Result<(), libsql::Error> {
    Release::migrate(conn).await?;
    Party::migrate(conn).await?;
    Work::migrate(conn).await?;
    Share::migrate(conn).await?;
    // conn.execute("CREATE INDEX idx_party ON parties(id)")
    Ok(())
}
