use crate::bwarm::interface::BwarmEntry;
use csv::StringRecord;
use libsql::params;
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

    async fn migrate(conn: &libsql::Connection) -> Result<(), libsql::Error> {
        let sql = "
        CREATE TABLE IF NOT EXISTS works (
           id TEXT PRIMARY KEY NOT NULL,
           title TEXT NOT NULL,
           duration_ms REAL,
           iswc TEXT,
           in_dispute INTEGER NOT NULL DEFAULT 0,
           alt_id INTEGER,
           is_arrangement INTEGER NOT NULL,
           territory TEXT
        )";
        _ = conn.execute(sql, params!()).await?;
        Ok(())
    }

    async fn prepare(conn: &libsql::Connection) -> Result<libsql::Statement, libsql::Error> {
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
        conn.prepare(sql).await
    }

    async fn insert_from_csv(
        fields: &StringRecord,
        stmt: &mut libsql::Statement,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let x = Work {
            id: &fields[0],
            iswc: fields.get(1),
            title: &fields[2],
            duration_ms: fields[6].parse::<f64>().ok(),
            in_dispute: &fields[7] == "TRUE",
            alt_id: fields[10].parse().ok(),
            is_arrangement: &fields[9] == "TRUE",
            territory: fields.get(8),
        };
        _ = stmt
            .execute(params!(
                x.id,
                x.title,
                x.duration_ms,
                x.iswc,
                (x.in_dispute as i64),
                x.alt_id,
                (x.is_arrangement as i64),
                x.territory,
            ))
            .await?;
        Ok(())
    }
}
