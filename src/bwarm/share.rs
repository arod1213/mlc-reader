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
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Share {
    #[serde(rename = "#MusicalWorkRightShareRecordId")]
    pub id: String,
    #[serde(rename = "MusicalWorkRecordId")]
    pub work_id: String,
    #[serde(rename = "PartyRecordId")]
    pub party_id: i64,
    #[serde(rename = "PartyRole")]
    pub role: PartyRole,
    #[serde(rename = "RightShareType")]
    pub share_type: Option<String>,
    #[serde(rename = "RightsType")]
    pub rights_type: Option<String>,
    #[serde(rename = "RightSharePercentage")]
    pub share: Option<f64>,
    #[serde(rename = "TerritoryCode")]
    pub territory: String,
    #[serde(rename = "PrecedingMusicalWorkRightShareRecordId")]
    pub preceding_id: Option<String>,
}

impl BwarmEntry for Share {
    fn filename() -> String {
        "musicalworkrightshares.tsv".into()
    }

    async fn migrate(conn: &libsql::Connection) -> Result<(), libsql::Error> {
        let sql = "
        CREATE TABLE IF NOT EXISTS shares (
           id TEXT PRIMARY KEY NOT NULL,
           work_id TEXT NOT NULL REFERENCES works(id),
           party_id INTEGER NOT NULL REFERENCES parties(id),
           role TEXT NOT NULL,
           share_type TEXT,
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
           share_type,
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
           ?8,
           ?9
        )";
        conn.prepare(sql).await
    }

    async fn insert(&self, stmt: &mut libsql::Statement) -> Result<(), libsql::Error> {
        _ = stmt
            .execute(params!(
                self.id.clone(),
                self.work_id.clone(),
                self.party_id,
                self.role.to_string(),
                self.share_type.clone(),
                self.rights_type.clone(),
                self.share.unwrap_or_default(),
                self.territory.clone(),
                self.preceding_id.clone(),
            ))
            .await?;
        Ok(())
    }
}
