use libsql::params;
use serde::{Deserialize, Serialize};

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
        ) VALUES (
           ?1,
           ?2,
           ?3,
           ?4,
           ?5,
           ?6,
           ?7,
           ?8,
           ?9,
           ?10
        )
        ";
        conn.prepare(sql).await
    }

    async fn insert(&self, stmt: &mut libsql::Statement) -> Result<(), libsql::Error> {
        _ = stmt
            .execute(params!(
                self.id,
                self.email.as_deref(),
                self.isni.as_deref(),
                self.cisac_id.as_deref(),
                self.dpid.as_deref(),
                self.ipi,
                self.contact_name.as_deref(),
                self.full_name.as_str(),
                self.first_name.as_deref(),
                self.last_name.as_deref(),
            ))
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
