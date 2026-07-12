use crate::bwarm::interface::BwarmEntry;
use libsql::params;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Release {
    #[serde(rename = "#ReleaseRecordId")]
    pub id: i64,
    #[serde(rename = "ReleaseTitle")]
    pub title: String,
    #[serde(rename = "DisplayArtistName")]
    pub artist_name: String,
    #[serde(rename = "LabelName")]
    pub label_name: String,
    #[serde(rename = "DistributorName")]
    pub distro_name: String,
}

impl BwarmEntry for Release {
    fn filename() -> String {
        "releases.tsv".into()
    }

    async fn migrate(conn: &libsql::Connection) -> Result<(), libsql::Error> {
        let sql = "
        CREATE TABLE IF NOT EXISTS releases (
           id INTEGER PRIMARY KEY NOT NULL,
           title TEXT NOT NULL,
           artist_name TEXT NOT NULL,
           distro_name TEXT NOT NULL,
           label_name TEXT NOT NULL
        )";
        _ = conn.execute(sql, params!()).await?;
        Ok(())
    }

    async fn prepare(conn: &libsql::Connection) -> Result<libsql::Statement, libsql::Error> {
        let sql = "
        INSERT INTO releases (
           id,
           title,
           artist_name,
           distro_name,
           label_name
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
                self.id,
                self.title.as_str(),
                self.artist_name.as_str(),
                self.distro_name.as_str(),
                self.label_name.as_str(),
            ))
            .await?;
        Ok(())
    }
}
