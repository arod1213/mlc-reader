use libsql::params;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::bwarm::interface::BwarmEntry;

#[derive(Debug, Serialize, Deserialize)]
pub struct Share {
    #[serde(rename = "#MusicalWorkRightShareRecordId")]
    pub id: String,
    #[serde(rename = "MusicalWorkRecordId")]
    pub work_id: String,
    #[serde(rename = "PartyRecordId")]
    pub party_id: i64,
    #[serde(rename = "PartyRole")]
    pub role: String,
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
        )
        SELECT
           json_extract(value, '$.id'),
           json_extract(value, '$.work_id'),
           json_extract(value, '$.party_id'),
           json_extract(value, '$.role'),
           json_extract(value, '$.share_type'),
           json_extract(value, '$.rights_type'),
           json_extract(value, '$.share'),
           json_extract(value, '$.territory'),
           json_extract(value, '$.preceding_id')
        FROM json_each(?1)";
        conn.prepare(sql).await
    }

    async fn insert_many(
        objects: &[Self],
        stmt: &mut libsql::Statement,
    ) -> Result<(), libsql::Error> {
        let rows = objects
            .iter()
            .map(|share| {
                json!({
                    "id": share.id.as_str(),
                    "work_id": share.work_id.as_str(),
                    "party_id": share.party_id,
                    "role": share.role.as_str(),
                    "share_type": share.share_type.as_deref(),
                    "rights_type": share.rights_type.as_deref(),
                    "share": share.share.unwrap_or_default(),
                    "territory": share.territory.as_str(),
                    "preceding_id": share.preceding_id.as_deref(),
                })
            })
            .collect::<Vec<_>>();

        _ = stmt
            .execute(params!(serde_json::to_string(&rows).unwrap()))
            .await?;
        Ok(())
    }
}
