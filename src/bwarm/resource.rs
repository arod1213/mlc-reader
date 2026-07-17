use libsql::params;
use musicmeta::isrc::Isrc;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::bwarm::interface::BwarmEntry;

#[derive(Debug, Serialize, Deserialize)]
pub struct Resource {
    // if unmatched add field
    #[serde(rename = "#ResourceRecordId", alias = "#UnmatchedResourceRecordId")]
    pub id: String,
    #[serde(rename = "OriginalDataProviderName")]
    pub data_provider: String,
    #[serde(rename = "ReleaseRecordId")]
    pub release_id: u64,
    #[serde(rename = "ResourceType")]
    pub resource_type: String,
    #[serde(rename = "ISRC")]
    pub isrc: Option<Isrc>,
    #[serde(rename = "Title")]
    pub title: String,

    #[serde(default)]
    pub is_matched: bool,
}
impl BwarmEntry for Resource {
    fn filename() -> String {
        "resources.tsv".into()
    }

    async fn migrate(conn: &libsql::Connection) -> Result<(), libsql::Error> {
        let sql = "
        CREATE TABLE IF NOT EXISTS resources (
           id TEXT PRIMARY KEY NOT NULL,
           data_provider TEXT NOT NULL,
           release_id INTEGER NOT NULL REFERENCES releases(id) ON DELETE CASCADE,
           isrc TEXT,
           title TEXT NOT NULL
        )";
        _ = conn.execute(sql, params!()).await?;
        Ok(())
    }

    async fn prepare(conn: &libsql::Connection) -> Result<libsql::Statement, libsql::Error> {
        let sql = "
        INSERT INTO resources (
           id,
           data_provider,
           release_id,
           isrc,
           title
        )
        SELECT
           json_extract(value, '$.id'),
           json_extract(value, '$.data_provider'),
           json_extract(value, '$.release_id'),
           json_extract(value, '$.isrc'),
           json_extract(value, '$.title')
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
            .map(|resource| {
                json!({
                    "id": resource.id.as_str(),
                    "data_provider": resource.data_provider.as_str(),
                    "release_id": resource.release_id,
                    "isrc": resource.isrc.as_ref().map(|x| x.to_string()),
                    "title": resource.title.as_str(),
                })
            })
            .collect::<Vec<_>>();

        _ = stmt
            .execute(params!(serde_json::to_string(&rows).unwrap()))
            .await?;
        Ok(())
    }
}
