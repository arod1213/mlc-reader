use std::{error::Error, fs::File};

use libsql::{Connection, Transaction};
use serde::de::DeserializeOwned;

use crate::bwarm::{
    interface::BwarmEntry,
    types::{Party, Release, Share, Work},
};

async fn save_object<T: BwarmEntry + DeserializeOwned>(
    tx: &Transaction,
) -> Result<(), Box<dyn Error>> {
    let fullpath = dirs::download_dir().unwrap().join(T::filename());

    let file = File::open(fullpath)?;
    let mut rdr = csv::ReaderBuilder::new()
        .delimiter(b'\t')
        .has_headers(true)
        .from_reader(file);

    let headers = rdr.headers()?.clone();
    dbg!(&headers);
    let mut stmt = T::prepare(tx).await?;
    for entry in rdr.records() {
        let x = entry?;
        let obj: T = x.deserialize(Some(&headers))?;
        obj.insert(&mut stmt).await?;
        stmt.reset();
    }
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

pub async fn migrate_from_bwarm_dump(conn: &Connection) {
    migrate(conn).await.unwrap();
    let tx = conn.transaction().await.unwrap();
    let res = async {
        save_object::<Release>(&tx).await?;
        save_object::<Party>(&tx).await?;
        save_object::<Work>(&tx).await?;
        save_object::<Share>(&tx).await?;
        Ok::<_, Box<dyn std::error::Error>>(())
    }
    .await;
    match res {
        Ok(_) => {
            tx.commit().await.unwrap();
        }
        Err(_) => {
            tx.rollback().await.unwrap();
        }
    }
}
