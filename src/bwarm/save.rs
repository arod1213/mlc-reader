use libsql::params;

use crate::bwarm::{
    interface::BwarmEntry,
    types::{Party, Release, Share, Work},
};

impl BwarmEntry for Release {
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
           label_name
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

    async fn insert(&self, stmt: &mut libsql::Statement) -> Result<(), libsql::Error> {
        _ = stmt
            .execute(params!(
                self.id,
                self.title.as_str(),
                self.artist_name.as_str(),
                self.distro_name.as_str(),
                self.label_name.as_str(),
            ))
            .await?;
        Ok(())
    }
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
           share_type TEXT NOT NULL,
           rights_type TEXT NOT NULL,
           share REAL NOT NULL,
           territory_code TEXT NOT NULL,
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
           territory_code,
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
                self.role.clone(),
                self.share_type.clone(),
                self.rights_type.clone(),
                self.share.unwrap_or_default(),
                self.territory_code.clone(),
                self.preceding_id.clone(),
            ))
            .await?;
        Ok(())
    }
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
           territory_code INTEGER
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
           territory_code
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
                self.territory.map(|x| x.code() as i64),
            ))
            .await?;
        Ok(())
    }
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
