use csv::StringRecord;
use libsql::params;
use serde::{Deserialize, Serialize};

use crate::bwarm::interface::BwarmEntry;

#[derive(Debug, Serialize, Deserialize)]
pub struct Resource<'a> {
    pub id: &'a str,
    pub data_provider: &'a str,
    pub release_id: u64,
    pub resource_type: &'a str,
    pub isrc: Option<&'a str>,
    pub title: &'a str,
}

impl Resource<'_> {
    pub fn from_csv<'r>(
        fields: &'r StringRecord,
    ) -> Result<Resource<'r>, Box<dyn std::error::Error>> {
        Ok(Resource {
            id: &fields[0],
            data_provider: &fields[14],
            release_id: fields[11].parse::<u64>()?,
            resource_type: &fields[1],
            isrc: fields.get(2),
            title: &fields[3],
        })
    }
}

impl BwarmEntry for Resource<'_> {
    fn filename() -> String {
        "resources.tsv".into()
    }

    async fn migrate(conn: &libsql::Connection) -> Result<(), libsql::Error> {
        let sql = "
        CREATE TABLE IF NOT EXISTS resources (
           id TEXT PRIMARY KEY NOT NULL,
           data_provider TEXT NOT NULL,
           release_id INTEGER NOT NULL REFERENCES releases(id) ON DELETE CASCADE,
           isrc TEXT,
           title TEXT NOT NULL
        )";
        _ = conn.execute(sql, params!()).await?;
        Ok(())
    }

    async fn prepare(conn: &libsql::Connection) -> Result<libsql::Statement, libsql::Error> {
        let sql = "
        INSERT INTO resources (
           id,
           data_provider,
           release_id,
           isrc,
           title
        ) VALUES (
           ?1,
           ?2,
           ?3,
           ?4,
           ?5
        )";
        let stmt = conn.prepare(sql).await?;
        Ok(stmt)
    }

    async fn insert_from_csv(
        fields: &StringRecord,
        stmt: &mut libsql::Statement,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let x = Resource {
            id: &fields[0],
            data_provider: &fields[14],
            release_id: fields[11].parse::<u64>()?,
            resource_type: &fields[1],
            isrc: fields.get(2),
            title: &fields[3],
        };
        _ = stmt
            .execute(params!(
                x.id,
                x.data_provider,
                x.release_id,
                x.isrc,
                x.title,
            ))
            .await?;
        Ok(())
    }
}
