use std::{fs::File, path::Path};

use libsql::{Connection, params};

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

pub async fn save_object<'r, T>(
    conn: &Connection,
    bwarm_dir: &Path,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
where
    T: 'r + BwarmEntry,
{
    let fullpath = bwarm_dir.join(T::filename());
    println!("about to save {:?}", fullpath);

    let file = File::open(fullpath)?;
    let mut rdr = csv::ReaderBuilder::new()
        .delimiter(b'\t')
        .flexible(true)
        .has_headers(true)
        .from_reader(file);

    let mut tx = conn.transaction().await?;
    let mut stmt = T::prepare(&tx).await?;
    let mut sum = 0;

    let batch_size = 1_000_000;
    for entry in rdr.records() {
        let entry = entry?;
        match T::insert_from_csv(&entry, &mut stmt).await {
            Ok(_) => {
                sum += 1;
                if sum % batch_size == 0 {
                    drop(stmt);
                    tx.commit().await?;
                    println!("@ {}", sum);

                    tx = conn.transaction().await?;
                    stmt = T::prepare(&tx).await?;
                }
            }
            Err(e) => {
                eprintln!("failed to save: {e}");
            }
        };
        stmt.reset();
    }

    drop(stmt);
    tx.commit().await?;
    println!("inserted {sum}");
    Ok(())
}

pub async fn migrate_schema(conn: &Connection) -> Result<(), libsql::Error> {
    Release::migrate(conn).await?;
    Resource::migrate(conn).await?;
    Party::migrate(conn).await?;
    Work::migrate(conn).await?;
    WorkResource::migrate(conn).await?;
    Share::migrate(conn).await?;
    // conn.execute("CREATE INDEX idx_party ON parties(id)")
    Ok(())
}
