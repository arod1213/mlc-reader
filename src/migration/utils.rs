use std::{fs::File, path::Path};

use rusqlite::{Connection, params};

use crate::bwarm::{
    interface::BwarmEntry, party::Party, release::Release, resource::Resource, share::Share,
    work_resource::WorkResource, works::Work,
};

pub fn disable_fk(conn: &Connection) -> Result<(), rusqlite::Error> {
    _ = conn.execute("PRAGMA foreign_keys = OFF", params!())?;
    Ok(())
}

pub fn setup_write_mode(conn: &Connection) -> Result<(), rusqlite::Error> {
    let mut stmt = conn.prepare("PRAGMA journal_mode = WAL")?;
    let mut rows = stmt.query([])?;
    while rows.next()?.is_some() {}

    conn.execute("PRAGMA synchronous = NORMAL", params!())?;
    Ok(())
}

pub fn save_object<'r, T>(
    conn: &mut Connection,
    bwarm_dir: &Path,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
where
    T: 'r + BwarmEntry,
{
    let fullpath = bwarm_dir.join(T::filename());
    log::debug!("about to save {:?}", fullpath);

    let file = File::open(fullpath)?;
    let mut rdr = csv::ReaderBuilder::new()
        .delimiter(b'\t')
        .flexible(true)
        .has_headers(true)
        .from_reader(file);

    let mut tx = conn.transaction()?;
    let mut stmt = T::prepare(&tx)?;
    let mut sum = 0;

    let batch_size = 1_000_000;
    for entry in rdr.records() {
        let entry = entry?;
        match T::insert_from_csv(&entry, &mut stmt) {
            Ok(_) => {
                sum += 1;
                if sum % batch_size == 0 {
                    drop(stmt);
                    tx.commit()?;
                    log::debug!("@ {}", sum);

                    tx = conn.transaction()?;
                    stmt = T::prepare(&tx)?;
                }
            }
            Err(e) => {
                log::error!("failed to save: {e}");
            }
        };
    }

    drop(stmt);
    tx.commit()?;
    log::info!("inserted {sum}");
    Ok(())
}

pub fn migrate_schema(conn: &Connection) -> Result<(), rusqlite::Error> {
    Release::migrate(conn)?;
    Resource::migrate(conn)?;
    Party::migrate(conn)?;
    Work::migrate(conn)?;
    WorkResource::migrate(conn)?;
    Share::migrate(conn)?;
    // conn.execute("CREATE INDEX idx_party ON parties(id)")
    Ok(())
}
