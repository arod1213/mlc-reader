use crate::bwarm::interface::BwarmEntry;
use csv::StringRecord;
use rusqlite::params;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Release<'a> {
    pub id: i64,
    pub title: &'a str,
    pub artist_name: Option<&'a str>,
    pub label_name: &'a str,
    pub release_date: Option<&'a str>,
    pub distro_name: &'a str,
}

impl<'a> BwarmEntry for Release<'a> {
    fn filename() -> String {
        "releases.tsv".into()
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
        f("id", "#ReleaseRecordId", 0)?;
        f("title", "ReleaseTitle", 2)?;
        f("artist_name", "DisplayArtistName", 5)?;
        f("label_name", "LabelName", 7)?;
        f("distro_name", "DistributorName", 8)?;
        f("release_date", "ReleaseDate", 9)?;
        Ok(())
    }

    fn migrate(conn: &rusqlite::Connection) -> Result<(), rusqlite::Error> {
        let sql = "
        CREATE TABLE IF NOT EXISTS releases (
            id INTEGER PRIMARY KEY NOT NULL,
            title TEXT,
            artist_name TEXT,
            distro_name TEXT NOT NULL,
            -- UNIX Timestamp
            release_date INTEGER,
            label_name TEXT NOT NULL
        )";
        _ = conn.execute(sql, params!())?;
        Ok(())
    }

    fn prepare<'c>(
        conn: &'c rusqlite::Connection,
    ) -> Result<rusqlite::Statement<'c>, rusqlite::Error> {
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
        let stmt = conn.prepare(sql)?;
        Ok(stmt)
    }

    fn insert_from_csv(
        fields: &StringRecord,
        stmt: &mut rusqlite::Statement,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let title = match fields.get(2) {
            None | Some("") => return Ok(false), // skip missing titles
            Some(x) => x,
        };
        let artist_name = match fields.get(5) {
            None | Some("") => None,
            Some(x) => Some(x),
        };

        let x = Release {
            id: fields.get(0).ok_or("missing id")?.parse::<i64>()?,
            title,
            artist_name,
            label_name: fields.get(7).ok_or("missing label name")?,
            release_date: fields.get(9),
            distro_name: fields.get(8).ok_or("missing distro name")?,
        };
        _ = stmt.execute(params!(
            x.id,
            x.title,
            x.artist_name,
            x.distro_name,
            x.release_date,
            x.label_name,
        ))?;
        Ok(true)
    }
}
