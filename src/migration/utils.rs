use std::{error::Error, fs::File, path::Path};

use libsql::{Connection, Transaction, params};
use serde::de::DeserializeOwned;

use crate::bwarm::{
    interface::BwarmEntry, party::Party, release::Release, resource::Resource, share::Share,
    work_resource::WorkResource, works::Work,
};
pub async fn disable_fk(conn: &Connection) -> Result<(), libsql::Error> {
    _ = conn.execute("PRAGMA foreign_keys = OFF", params!()).await?;
    Ok(())
}

pub async fn setup_write_mode(conn: &Connection) -> Result<(), libsql::Error> {
    let mut rows = conn.query("PRAGMA journal_mode = WAL", params!()).await?;
    while rows.next().await?.is_some() {}

    conn.execute("PRAGMA synchronous = NORMAL", params!())
        .await?;
    Ok(())
}

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

    let capacity = 500;
    let mut objects: Vec<T> = Vec::with_capacity(capacity);
    for entry in rdr.records() {
        let mut x = entry?;
        while x.len() < headers.len() {
            x.push_field("");
        }

        let obj = match x.deserialize::<T>(Some(&headers)) {
            Ok(x) => x,
            Err(e) => {
                eprintln!("error: {e}\nfrom: {:?}", x);
                continue;
            }
        };
        objects.push(obj);
        if objects.len() == capacity {
            T::insert_many(objects.as_slice(), &mut stmt).await?;
            objects.clear();
            stmt.reset();
        }
        sum += 1;
    }

    // flush remaining
    if !objects.is_empty() {
        T::insert_many(objects.as_slice(), &mut stmt).await?;
    }

    println!("INSERTED {}", sum);
    Ok(())
}

pub async fn migrate_schema(conn: &Connection) -> Result<(), libsql::Error> {
    Release::migrate(conn).await?;
    Resource::migrate(conn).await?;
    Party::migrate(conn).await?;
    Work::migrate(conn).await?;
    WorkResource::migrate(conn).await?;
    Share::migrate(conn).await?;
    Ok(())
}
