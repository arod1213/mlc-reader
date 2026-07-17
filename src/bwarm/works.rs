use crate::bwarm::interface::BwarmEntry;
use libsql::params;
use serde::{Deserialize, Deserializer, Serialize, de};
use serde_json::json;

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
        )
        SELECT
           json_extract(value, '$.id'),
           json_extract(value, '$.title'),
           json_extract(value, '$.duration_ms'),
           json_extract(value, '$.iswc'),
           json_extract(value, '$.in_dispute'),
           json_extract(value, '$.alt_id'),
           json_extract(value, '$.is_arrangement'),
           json_extract(value, '$.territory')
        FROM json_each(?1)";
        conn.prepare(sql).await
    }

    async fn insert_many(
        objects: &[Self],
        stmt: &mut libsql::Statement,
    ) -> Result<(), libsql::Error> {
        let rows = objects
            .iter()
            .map(|work| {
                json!({
                    "id": work.id.as_str(),
                    "title": work.title.as_str(),
                    "duration_ms": work.duration_ms,
                    "iswc": work.iswc.as_deref(),
                    "in_dispute": work.in_dispute as i64,
                    "alt_id": work.alt_id,
                    "is_arrangement": work.is_arrangement as i64,
                    "territory": work.territory.as_deref(),
                })
            })
            .collect::<Vec<_>>();

        _ = stmt
            .execute(params!(serde_json::to_string(&rows).unwrap()))
            .await?;
        Ok(())
    }
}
