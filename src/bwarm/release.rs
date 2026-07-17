use crate::bwarm::interface::BwarmEntry;
use chrono::{NaiveDate, NaiveTime};
use libsql::params;
use serde::{Deserialize, Serialize};
use serde_json::json;

fn date_yyyy_mm_dd<'de, D>(deserializer: D) -> Result<Option<NaiveDate>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let text = String::deserialize(deserializer)?;
    // NaiveDate::parse_from_str(&text, "%y-%m-%d").map_err(|e| de::Error::custom(e.to_string()))
    Ok(NaiveDate::parse_from_str(&text, "%y-%m-%d").ok())
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
        )
        SELECT
           json_extract(value, '$.id'),
           json_extract(value, '$.title'),
           json_extract(value, '$.artist_name'),
           json_extract(value, '$.distro_name'),
           json_extract(value, '$.release_date'),
           json_extract(value, '$.label_name')
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
            .map(|release| {
                let timestamp: Option<i64> = release
                    .release_date
                    .map(|x| x.and_time(NaiveTime::default()).and_utc().timestamp());

                json!({
                    "id": release.id,
                    "title": release.title.as_str(),
                    "artist_name": release.artist_name.as_str(),
                    "distro_name": release.distro_name.as_str(),
                    "release_date": timestamp,
                    "label_name": release.label_name.as_str(),
                })
            })
            .collect::<Vec<_>>();

        _ = stmt
            .execute(params!(serde_json::to_string(&rows).unwrap()))
            .await?;
        Ok(())
    }
}
