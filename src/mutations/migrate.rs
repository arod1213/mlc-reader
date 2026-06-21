use std::{error::Error, fs::File, path::Path};

use libsql::{Connection, Transaction};
use serde::de::DeserializeOwned;

use crate::bwarm::{
    interface::BwarmEntry,
    types::{Party, Release, Share, Work},
};

async fn save_object<T: BwarmEntry + DeserializeOwned>(
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

async fn migrate(conn: &Connection) -> Result<(), libsql::Error> {
    Release::migrate(conn).await?;
    Party::migrate(conn).await?;
    Work::migrate(conn).await?;
    Share::migrate(conn).await?;
    // conn.execute("CREATE INDEX idx_party ON parties(id)")
    Ok(())
}

pub async fn migrate_from_bwarm_dump(conn: &Connection, bwarm_dir: &Path) {
    migrate(conn).await.expect("failed to migrate");
    let tx = conn
        .transaction()
        .await
        .expect("failed to setup transaction");
    let res = async {
        save_object::<Release>(&tx, bwarm_dir).await?;
        save_object::<Party>(&tx, bwarm_dir).await?;
        save_object::<Work>(&tx, bwarm_dir).await?;
        save_object::<Share>(&tx, bwarm_dir).await?;
        Ok::<_, Box<dyn std::error::Error>>(())
    }
    .await;
    match res {
        Ok(_) => {
            tx.commit().await.expect("failed to commit");
            println!("inserted BWARM");
        }
        Err(e) => {
            tx.rollback().await.expect("failed to rollback");
            println!("failed: {}", e);
        }
    }
}
