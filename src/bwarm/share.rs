use csv::StringRecord;
use libsql::params;
use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};

use crate::bwarm::interface::BwarmEntry;

#[derive(Debug, Serialize, Deserialize, Display, EnumString)]
#[serde(rename_all = "PascalCase")]
#[strum(serialize_all = "snake_case")]
pub enum PartyRole {
    OriginalPublisher,
    RightsAdministrator,
    Composer,
    SubPublisher,
    SubstitutedPublisher,
    Arranger,
    ComposerLyricist,
    Lyricist,
    SubLyricist,
    Translator,
    Adapter,
    SubArranger,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Share<'a> {
    pub id: &'a str,
    pub work_id: &'a str,
    pub party_id: i64,
    pub role: &'a str,
    pub rights_type: Option<&'a str>,
    pub share: Option<f64>,
    pub territory: &'a str,
    pub preceding_id: Option<&'a str>,
}

impl BwarmEntry for Share<'_> {
    fn filename() -> String {
        "musicalworkrightshares.tsv".into()
    }

    async fn migrate(conn: &libsql::Connection) -> Result<(), libsql::Error> {
        let sql = "
        CREATE TABLE IF NOT EXISTS shares (
           id TEXT NOT NULL,
           work_id TEXT NOT NULL REFERENCES works(id),
           party_id INTEGER NOT NULL REFERENCES parties(id),
           role TEXT NOT NULL,
           rights_type TEXT,
           share REAL NOT NULL,
           territory TEXT NOT NULL,
           preceding_id TEXT
        )";
        _ = conn.execute(sql, params!()).await?;
        Ok(())
    }

    async fn prepare(conn: &libsql::Connection) -> Result<libsql::Statement, libsql::Error> {
        let sql = "
        INSERT INTO shares (
           id,
           work_id,
           party_id,
           role,
           rights_type,
           share,
           territory,
           preceding_id
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
        let x = Share {
            id: fields.get(0).ok_or("missing id")?,
            work_id: fields.get(1).ok_or("missing work id")?,
            party_id: fields.get(2).ok_or("missing party id")?.parse::<i64>()?,
            role: fields.get(3).ok_or("missing role")?,
            share: fields.get(4).and_then(|x| x.parse::<f64>().ok()),
            rights_type: fields.get(5),
            territory: fields.get(7).ok_or("missing territory")?,
            preceding_id: fields.get(8),
        };
        _ = stmt
            .execute(params!(
                x.id,
                x.work_id,
                x.party_id,
                x.role,
                x.rights_type,
                x.share.unwrap_or_default(),
                x.territory,
                x.preceding_id,
            ))
            .await?;
        Ok(())
    }
}
