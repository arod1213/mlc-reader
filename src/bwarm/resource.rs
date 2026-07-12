use libsql::params;
use musicmeta::isrc::Isrc;
use serde::{Deserialize, Serialize};

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
    pub isrc: Isrc,
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
           isrc TEXT NOT NULL,
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
        ) VALUES (
           ?1,
           ?2,
           ?3,
           ?4,
           ?5
        )";
        let stmt = conn.prepare(sql).await?;
        Ok(stmt)
    }

    async fn insert(&self, stmt: &mut libsql::Statement) -> Result<(), libsql::Error> {
        _ = stmt
            .execute(params!(
                self.id.clone(),
                self.data_provider.clone(),
                self.release_id,
                self.isrc.to_string(),
                self.title.clone(),
            ))
            .await?;
        Ok(())
    }
}
