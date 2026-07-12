use libsql::params;
use serde::{Deserialize, Serialize};

use crate::bwarm::interface::BwarmEntry;

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkResource {
    #[serde(rename = "#LinkRecordId")]
    pub id: u64,
    #[serde(rename = "MusicalWorkRecordId")]
    pub work_id: String,
    #[serde(rename = "ResourceRecordId")]
    pub resource_id: String,
}

impl BwarmEntry for WorkResource {
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

    async fn insert(&self, stmt: &mut libsql::Statement) -> Result<(), libsql::Error> {
        _ = stmt
            .execute(params!(
                self.id.clone(),
                self.work_id.clone(),
                self.resource_id.clone(),
            ))
            .await?;
        Ok(())
    }
}
