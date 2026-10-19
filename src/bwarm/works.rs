use crate::bwarm::interface::BwarmEntry;
use csv::StringRecord;
use rusqlite::params;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Work<'a> {
    pub id: &'a str,
    pub title: &'a str,
    pub duration_ms: Option<f64>,
    pub iswc: Option<&'a str>,
    pub in_dispute: bool,
    pub alt_id: Option<i64>,
    pub is_arrangement: bool,
    /// String because the MLC and DDEX are idiots who cant conform to a simple standard
    pub territory: Option<&'a str>,
}

impl BwarmEntry for Work<'_> {
    fn filename() -> String {
        "musicalworks.tsv".into()
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
        f("id", "#MusicalWorkRecordId", 0)?;
        f("iswc", "ISWC", 1)?;
        f("title", "MusicalWorkTitle", 2)?;
        f("duration_ms", "NominalDuration", 6)?;
        f("in_dispute", "HasRightShareInDispute", 7)?;
        f("territory", "TerritoryOfPublicDomain", 8)?;
        f("is_arrangement", "IsArrangementOfTraditionalWork", 9)?;
        f(
            "alt_id",
            "AlternativeMusicalWorkIdForUsStatutoryReversion",
            10,
        )?;
        Ok(())
    }

    fn migrate(conn: &rusqlite::Connection) -> Result<(), rusqlite::Error> {
        let sql = "
        CREATE TABLE IF NOT EXISTS works (
           id TEXT NOT NULL,
           title TEXT NOT NULL,
           duration_ms REAL,
           iswc TEXT,
           in_dispute INTEGER NOT NULL DEFAULT 0,
           alt_id INTEGER,
           is_arrangement INTEGER NOT NULL,
           territory TEXT
        )";
        _ = conn.execute(sql, params!())?;
        Ok(())
    }

    fn prepare<'c>(
        conn: &'c rusqlite::Connection,
    ) -> Result<rusqlite::Statement<'c>, rusqlite::Error> {
        let sql = "
        INSERT INTO works (
           id,
           title,
           duration_ms,
           iswc,
           in_dispute,
           alt_id,
           is_arrangement,
           territory
        ) VALUES (
           ?1,
           ?2,
           ?3,
           ?4,
           ?5,
           ?6,
           ?7,
           ?8
        )";
        conn.prepare(sql)
    }

    fn insert_from_csv(
        fields: &StringRecord,
        stmt: &mut rusqlite::Statement,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let x = Work {
            id: fields.get(0).ok_or("missing id")?,
            iswc: fields.get(1),
            title: fields.get(2).ok_or("missing title")?,
            duration_ms: fields.get(6).and_then(|x| x.parse::<f64>().ok()),
            in_dispute: fields.get(7).ok_or("missing dispute flag")? == "TRUE",
            alt_id: fields.get(10).and_then(|x| x.parse().ok()),
            is_arrangement: fields.get(9).map(|x| x == "TRUE").unwrap_or_default(),
            territory: fields.get(8),
        };
        _ = stmt.execute(params!(
            x.id,
            x.title,
            x.duration_ms,
            x.iswc,
            (x.in_dispute as i64),
            x.alt_id,
            (x.is_arrangement as i64),
            x.territory,
        ))?;
        Ok(true)
    }
}
