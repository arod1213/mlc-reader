use crate::bwarm::interface::BwarmEntry;
use csv::StringRecord;
use libsql::params;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Release<'a> {
    pub id: i64,
    pub title: &'a str,
    pub artist_name: &'a str,
    pub label_name: &'a str,
    pub release_date: Option<&'a str>,
    pub distro_name: &'a str,
}

impl<'a> BwarmEntry for Release<'a> {
    fn filename() -> String {
        "releases.tsv".into()
    }

    async fn migrate(conn: &libsql::Connection) -> Result<(), libsql::Error> {
        let sql = "
        CREATE TABLE IF NOT EXISTS releases (
            id INTEGER PRIMARY KEY NOT NULL,
            title TEXT NOT NULL,
            artist_name TEXT NOT NULL,
            distro_name TEXT NOT NULL,
            -- UNIX Timestamp
            release_date INTEGER,
            label_name TEXT NOT NULL
        )";
        _ = conn.execute(sql, params!()).await?;
        Ok(())
    }

    async fn prepare(conn: &libsql::Connection) -> Result<libsql::Statement, libsql::Error> {
        let sql = "
        INSERT INTO releases (
           id,
           title,
           artist_name,
           distro_name,
           release_date,
           label_name
        ) VALUES (
           ?1,
           ?2,
           ?3,
           ?4,
           ?5,
           ?6
        )";
        let stmt = conn.prepare(sql).await?;
        Ok(stmt)
    }

    async fn insert_from_csv(
        fields: &StringRecord,
        stmt: &mut libsql::Statement,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let x = Release {
            id: fields[0].parse::<i64>()?,
            title: &fields[2],
            artist_name: &fields[5],
            label_name: &fields[7],
            release_date: fields.get(9),
            distro_name: &fields[8],
        };
        _ = stmt
            .execute(params!(
                x.id,
                x.title,
                x.artist_name,
                x.distro_name,
                x.release_date,
                x.label_name,
            ))
            .await?;
        Ok(())
    }
}
