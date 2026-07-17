use libsql::params;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::bwarm::interface::BwarmEntry;

#[derive(Debug, Serialize, Deserialize)]
pub struct Party {
    #[serde(rename = "#PartyRecordId")]
    pub id: i64,
    #[serde(rename = "EmailAddress", default)]
    pub email: Option<String>,
    #[serde(rename = "ISNI", default)]
    pub isni: Option<String>,
    #[serde(rename = "CisacID", alias = "CisacSocietyId", default)]
    pub cisac_id: Option<String>,
    #[serde(rename = "DPID", default)]
    pub dpid: Option<String>,
    #[serde(rename = "IpiNameNumber", default)]
    pub ipi: Option<i64>,
    #[serde(rename = "ContactName", default)]
    pub contact_name: Option<String>,
    #[serde(rename = "FullName")]
    pub full_name: String,
    #[serde(rename = "KeyName", default)]
    pub last_name: Option<String>,
    #[serde(rename = "Prefix", alias = "NamesBeforeKeyName", default)]
    pub first_name: Option<String>,
}

impl BwarmEntry for Party {
    fn filename() -> String {
        "parties.tsv".into()
    }

    async fn prepare(conn: &libsql::Connection) -> Result<libsql::Statement, libsql::Error> {
        let sql = "
        INSERT INTO parties (
           id,
           email,
           isni,
           cisac_id,
           dpid,
           ipi,
           contact_name,
           full_name,
           first_name,
           last_name
        )
        SELECT
           json_extract(value, '$.id'),
           json_extract(value, '$.email'),
           json_extract(value, '$.isni'),
           json_extract(value, '$.cisac_id'),
           json_extract(value, '$.dpid'),
           json_extract(value, '$.ipi'),
           json_extract(value, '$.contact_name'),
           json_extract(value, '$.full_name'),
           json_extract(value, '$.first_name'),
           json_extract(value, '$.last_name')
        FROM json_each(?1)";
        conn.prepare(sql).await
    }

    async fn insert_many(
        objects: &[Self],
        stmt: &mut libsql::Statement,
    ) -> Result<(), libsql::Error> {
        let rows = objects
            .iter()
            .map(|party| {
                json!({
                    "id": party.id,
                    "email": party.email.as_deref(),
                    "isni": party.isni.as_deref(),
                    "cisac_id": party.cisac_id.as_deref(),
                    "dpid": party.dpid.as_deref(),
                    "ipi": party.ipi,
                    "contact_name": party.contact_name.as_deref(),
                    "full_name": party.full_name.as_str(),
                    "first_name": party.first_name.as_deref(),
                    "last_name": party.last_name.as_deref(),
                })
            })
            .collect::<Vec<_>>();

        _ = stmt
            .execute(params!(serde_json::to_string(&rows).unwrap()))
            .await?;
        Ok(())
    }

    async fn migrate(conn: &libsql::Connection) -> Result<(), libsql::Error> {
        let sql = "
        CREATE TABLE IF NOT EXISTS parties (
           id INTEGER PRIMARY KEY NOT NULL,
           email TEXT,
           isni TEXT,
           cisac_id TEXT,
           dpid TEXT,
           pro INTEGER,
           role TEXT,
           ipi INTEGER,
           contact_name TEXT,
           full_name TEXT NOT NULL,
           first_name TEXT,
           last_name TEXT
        )";
        _ = conn.execute(sql, params!()).await?;
        Ok(())
    }
}
