use csv::StringRecord;
use libsql::params;
use serde::{Deserialize, Serialize};

use crate::bwarm::interface::BwarmEntry;

#[derive(Debug, Serialize, Deserialize)]
pub struct Party<'a> {
    pub id: i64,
    pub email: Option<&'a str>,
    pub isni: Option<&'a str>,
    pub cisac_id: Option<&'a str>,
    pub dpid: Option<&'a str>,
    pub ipi: Option<i64>,
    pub contact_name: Option<&'a str>,
    pub full_name: &'a str,
    pub last_name: Option<&'a str>,
    pub first_name: Option<&'a str>,
}

impl BwarmEntry for Party<'_> {
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

    async fn insert_from_csv(
        fields: &StringRecord,
        stmt: &mut libsql::Statement,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let x = Party {
            id: fields.get(0).ok_or("missing id")?.parse::<i64>()?,
            isni: fields.get(1),
            ipi: fields.get(2).and_then(|x| x.parse::<i64>().ok()),
            email: fields.get(10),
            cisac_id: fields.get(3),
            dpid: fields.get(4),
            contact_name: fields.get(9),
            full_name: fields.get(5).ok_or("missing full name")?,
            last_name: fields.get(7),
            first_name: fields.get(6),
        };
        _ = stmt
            .execute(params!(
                x.id,
                x.email,
                x.isni,
                x.cisac_id,
                x.dpid,
                x.ipi,
                x.contact_name,
                x.full_name,
                x.first_name,
                x.last_name,
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
