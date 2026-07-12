use crate::bwarm::interface::BwarmEntry;
use libsql::params;
use serde::{Deserialize, Deserializer, Serialize, de};

fn named_bool<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    let res = String::deserialize(deserializer)?;
    match res.trim().to_uppercase().as_str() {
        "TRUE" => Ok(true),
        "FALSE" => Ok(false),
        _ => Err(de::Error::custom(format!("invalid bool {res}"))),
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Work {
    #[serde(rename = "#MusicalWorkRecordId")]
    pub id: String,
    #[serde(rename = "MusicalWorkTitle")]
    pub title: String,
    #[serde(rename = "NominalDuration", default)]
    pub duration_ms: Option<f64>,
    #[serde(rename = "ISWC")]
    pub iswc: Option<String>,
    #[serde(rename = "HasRightShareInDispute", deserialize_with = "named_bool")]
    pub in_dispute: bool,
    #[serde(rename = "AlternativeMusicalWorkIdForUsStatutoryReversion")]
    pub alt_id: Option<i64>,
    #[serde(
        rename = "IsArrangementOfTraditionalWork",
        deserialize_with = "named_bool"
    )]
    pub is_arrangement: bool,
    #[serde(rename = "TerritoryOfPublicDomain")]
    /// String because the MLC and DDEX are idiots who cant conform to a simple standard
    pub territory: Option<String>,
}

impl BwarmEntry for Work {
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

    async fn insert(&self, stmt: &mut libsql::Statement) -> Result<(), libsql::Error> {
        _ = stmt
            .execute(params!(
                self.id.clone(),
                self.title.clone(),
                self.duration_ms,
                self.iswc.clone(),
                (self.in_dispute as i64),
                self.alt_id,
                (self.is_arrangement as i64),
                self.territory.as_deref(),
            ))
            .await?;
        Ok(())
    }
}
