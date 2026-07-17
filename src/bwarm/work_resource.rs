use libsql::params;
use serde::{Deserialize, Serialize};
use serde_json::json;

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
        )
        SELECT
           json_extract(value, '$.id'),
           json_extract(value, '$.work_id'),
           json_extract(value, '$.resource_id')
        FROM json_each(?1)";
        let stmt = conn.prepare(sql).await?;
        Ok(stmt)
    }

    async fn insert_many(
        objects: &[Self],
        stmt: &mut libsql::Statement,
    ) -> Result<(), libsql::Error> {
        let rows = objects
            .iter()
            .map(|work_resource| {
                json!({
                    "id": work_resource.id,
                    "work_id": work_resource.work_id.as_str(),
                    "resource_id": work_resource.resource_id.as_str(),
                })
            })
            .collect::<Vec<_>>();

        _ = stmt
            .execute(params!(serde_json::to_string(&rows).unwrap()))
            .await?;
        Ok(())
    }
}
