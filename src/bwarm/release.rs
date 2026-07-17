use crate::bwarm::interface::BwarmEntry;
use chrono::{NaiveDate, NaiveTime};
use libsql::params;
use serde::{Deserialize, Serialize};

fn date_yyyy_mm_dd<'de, D>(deserializer: D) -> Result<Option<NaiveDate>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let text = String::deserialize(deserializer)?;
    // NaiveDate::parse_from_str(&text, "%y-%m-%d").map_err(|e| de::Error::custom(e.to_string()))
    Ok(NaiveDate::parse_from_str(&text, "%Y-%m-%d").ok())
}

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
    #[serde(rename = "ReleaseDate", deserialize_with = "date_yyyy_mm_dd")]
    pub release_date: Option<NaiveDate>,
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
            -- UNIX Timestamp
            release_date INTEGER,
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
           release_date,
           label_name
        ) VALUES (
           ?1,
           ?2,
           ?3,
           ?4,
           ?5,
           ?6
        )";
        let stmt = conn.prepare(sql).await?;
        Ok(stmt)
    }

    async fn insert(&self, stmt: &mut libsql::Statement) -> Result<(), libsql::Error> {
        let utc_time = self
            .release_date
            .map(|x: NaiveDate| x.and_time(NaiveTime::default()).and_utc().timestamp());

        _ = stmt
            .execute(params!(
                self.id,
                self.title.as_str(),
                self.artist_name.as_str(),
                self.distro_name.as_str(),
                utc_time,
                self.label_name.as_str(),
            ))
            .await?;
        Ok(())
    }
}
