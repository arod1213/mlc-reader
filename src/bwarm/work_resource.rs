use csv::StringRecord;
use libsql::params;
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

    async fn migrate(conn: &libsql::Connection) -> Result<(), libsql::Error> {
        let sql = "
        CREATE TABLE IF NOT EXISTS work_resources (
           id TEXT PRIMARY KEY NOT NULL,
           work_id TEXT NOT NULL REFERENCES works(id),
           resource_id TEXT NOT NULL REFERENCES resources(id)
        )";
        _ = conn.execute(sql, params!()).await?;
        Ok(())
    }

    async fn prepare(conn: &libsql::Connection) -> Result<libsql::Statement, libsql::Error> {
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
        let stmt = conn.prepare(sql).await?;
        Ok(stmt)
    }

    async fn insert_from_csv(
        fields: &StringRecord,
        stmt: &mut libsql::Statement,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let x = WorkResource {
            id: fields.get(0).ok_or("missing id")?.parse::<u64>()?,
            work_id: fields.get(1).ok_or("missing work id")?,
            resource_id: fields.get(2).ok_or("missing resource id")?,
        };
        _ = stmt
            .execute(params!(x.id, x.work_id, x.resource_id,))
            .await?;
        Ok(())
    }
}
