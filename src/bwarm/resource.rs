use csv::StringRecord;
use rusqlite::params;
use serde::{Deserialize, Serialize};

use crate::bwarm::interface::BwarmEntry;

#[derive(Debug, Serialize, Deserialize)]
pub struct Resource<'a> {
    pub id: &'a str,
    pub data_provider: &'a str,
    pub release_id: Option<u64>,
    pub resource_type: &'a str,
    pub isrc: Option<&'a str>,
    pub title: &'a str,
}

impl BwarmEntry for Resource<'_> {
    fn filename() -> String {
        "resources.tsv".into()
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
        f("id", "#ResourceRecordId", 0)?;
        f("resource_type", "ResourceType", 1)?;
        f("isrc", "ISRC", 2)?;
        f("title", "Title", 3)?;
        f("release_id", "ReleaseRecordId", 11)?;
        f("data_provider", "OriginalDataProviderName", 14)?;
        Ok(())
    }

    fn migrate(conn: &rusqlite::Connection) -> Result<(), rusqlite::Error> {
        let sql = "
        CREATE TABLE IF NOT EXISTS resources (
           -- id TEXT PRIMARY KEY NOT NULL,
           id TEXT NOT NULL,
           data_provider TEXT NOT NULL,
           release_id INTEGER REFERENCES releases(id) ON DELETE CASCADE,
           isrc TEXT,
           title TEXT NOT NULL
        )";
        _ = conn.execute(sql, params!())?;
        Ok(())
    }

    fn prepare<'c>(
        conn: &'c rusqlite::Connection,
    ) -> Result<rusqlite::Statement<'c>, rusqlite::Error> {
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
        let stmt = conn.prepare(sql)?;
        Ok(stmt)
    }

    fn insert_from_csv(
        fields: &StringRecord,
        stmt: &mut rusqlite::Statement,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let release_id = match fields.get(11) {
            Some(x) => {
                if x.is_empty() {
                    None
                } else {
                    Some(x.parse::<u64>()?)
                }
            }
            None => None,
        };

        let x = Resource {
            id: fields.get(0).ok_or("missing id")?,
            data_provider: fields.get(14).ok_or("missing data provider")?,
            release_id,
            resource_type: fields.get(1).ok_or("missing resource type")?,
            isrc: fields.get(2),
            title: fields.get(3).ok_or("missing title")?,
        };
        _ = stmt.execute(params!(
            x.id,
            x.data_provider,
            x.release_id.map(|x| x as i64),
            x.isrc,
            x.title,
        ))?;
        Ok(())
    }
}
