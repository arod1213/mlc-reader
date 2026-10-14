use csv::StringRecord;
use rusqlite::params;
use serde::{Deserialize, Serialize};

use crate::bwarm::interface::BwarmEntry;

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkResource<'a> {
    pub id: u64,
    pub work_id: &'a str,
    pub resource_id: &'a str,
}

impl BwarmEntry for WorkResource<'_> {
    fn filename() -> String {
        "workresourcelinks.tsv".into()
    }

    fn validate_headers(headers: &StringRecord) -> Result<(), String> {
        let f = |field, header, idx| -> Result<(), String> {
            let Some(x) = headers.get(idx) else {
                return Err(format!("could not get column {idx}"));
            };
            if x != header {
                return Err(format!("{field} ({header}) is not at column {idx}"));
            }
            Ok(())
        };
        f("id", "#LinkRecordId", 0)?;
        f("work_id", "MusicalWorkRecordId", 1)?;
        f("resource_id", "ResourceRecordId", 2)?;
        Ok(())
    }

    fn migrate(conn: &rusqlite::Connection) -> Result<(), rusqlite::Error> {
        let sql = "
        CREATE TABLE IF NOT EXISTS work_resources (
           id TEXT PRIMARY KEY NOT NULL,
           work_id TEXT NOT NULL REFERENCES works(id),
           resource_id TEXT NOT NULL REFERENCES resources(id)
        )";
        _ = conn.execute(sql, params!())?;
        Ok(())
    }

    fn prepare<'c>(
        conn: &'c rusqlite::Connection,
    ) -> Result<rusqlite::Statement<'c>, rusqlite::Error> {
        let sql = "
        INSERT INTO work_resources (
           id,
           work_id,
           resource_id
        ) VALUES (
           ?1,
           ?2,
           ?3
        )";
        let stmt = conn.prepare(sql)?;
        Ok(stmt)
    }

    fn insert_from_csv(
        fields: &StringRecord,
        stmt: &mut rusqlite::Statement,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let x = WorkResource {
            id: fields.get(0).ok_or("missing id")?.parse::<u64>()?,
            work_id: fields.get(1).ok_or("missing work id")?,
            resource_id: fields.get(2).ok_or("missing resource id")?,
        };
        _ = stmt.execute(params!(x.id as i64, x.work_id, x.resource_id,))?;
        Ok(())
    }
}
