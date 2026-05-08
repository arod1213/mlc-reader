use crate::bwarm::{
    interface::BwarmEntry,
    types::{Party, Share, Work},
};

impl BwarmEntry for Share {
    fn filename() -> String {
        "musicalworkrightshares.tsv".into()
    }

    fn migrate(conn: &sqlite::Connection) -> Result<(), sqlite::Error> {
        let sql = "
        CREATE TABLE IF NOT EXISTS shares (
           id TEXT PRIMARY KEY NOT NULL,
           work_id TEXT NOT NULL,
           party_id INTEGER NOT NULL,
           role TEXT NOT NULL,
           share_type TEXT NOT NULL,
           rights_type TEXT NOT NULL,
           share REAL NOT NULL,
           territory_code TEXT NOT NULL,
           preceding_id TEXT
        )";
        conn.execute(sql)
    }

    fn stmt<'a>(conn: &'a sqlite::Connection) -> Result<sqlite::Statement<'a>, sqlite::Error> {
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
           @id,
           @work_id,
           @party_id,
           @role,
           @share_type,
           @rights_type,
           @share,
           @territory_code,
           @preceding_id
        )";
        conn.prepare(sql)
    }

    fn bind(&self, stmt: &mut sqlite::Statement) -> Result<(), sqlite::Error> {
        stmt.bind::<&[(_, sqlite::Value)]>(&[
            ("@id", self.id.clone().into()),
            ("@work_id", self.work_id.clone().into()),
            ("@party_id", self.party_id.into()),
            ("@role", self.role.clone().into()),
            ("@share_type", self.share_type.clone().into()),
            ("@rights_type", self.rights_type.clone().into()),
            ("@share", self.share.unwrap_or_default().into()),
            ("@territory_code", self.territory_code.clone().into()),
            ("@preceding_id", self.preceding_id.clone().into()),
        ])
    }
}
impl BwarmEntry for Work {
    fn filename() -> String {
        "musicalworks.tsv".into()
    }

    fn migrate(conn: &sqlite::Connection) -> Result<(), sqlite::Error> {
        let sql = "
        CREATE TABLE IF NOT EXISTS works (
           id TEXT PRIMARY KEY NOT NULL,
           title TEXT NOT NULL,
           duration_ms REAL NOT NULL,
           iswc TEXT,
           in_dispute INTEGER NOT NULL,
           alt_id INTEGER,
           is_arrangement INTEGER NOT NULL,
           territory TEXT
        )";
        conn.execute(sql)
    }

    fn stmt<'a>(conn: &'a sqlite::Connection) -> Result<sqlite::Statement<'a>, sqlite::Error> {
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
        ) VALUES (
           @id,
           @title,
           @duration_ms,
           @iswc,
           @in_dispute,
           @alt_id,
           @is_arrangement,
           @territory
        )";
        conn.prepare(sql)
    }

    fn bind(&self, stmt: &mut sqlite::Statement) -> Result<(), sqlite::Error> {
        stmt.bind::<&[(_, sqlite::Value)]>(&[
            ("@id", self.id.clone().into()),
            ("@title", self.title.clone().into()),
            ("@duration_ms", self.duration_ms.into()),
            ("@iswc", self.iswc.clone().into()),
            ("@in_dispute", (self.in_dispute as i64).into()),
            ("@alt_id", self.alt_id.into()),
            ("@is_arrangement", (self.is_arrangement as i64).into()),
            ("@territory", self.territory.clone().into()),
        ])
    }
}

impl BwarmEntry for Party {
    fn filename() -> String {
        "parties.tsv".into()
    }

    fn stmt<'a>(conn: &'a sqlite::Connection) -> Result<sqlite::Statement<'a>, sqlite::Error> {
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
           key_name,
           prefix
        ) VALUES (
           @id,
           @email,
           @isni,
           @cisac_id,
           @dpid,
           @ipi,
           @contact_name,
           @full_name,
           @key_name,
           @prefix
        )
        ";
        conn.prepare(sql)
    }

    fn bind(&self, stmt: &mut sqlite::Statement) -> Result<(), sqlite::Error> {
        stmt.bind::<&[(_, sqlite::Value)]>(&[
            ("@id", self.id.into()),
            ("@email", self.email.clone().into()),
            ("@isni", self.isni.clone().into()),
            ("@cisac_id", self.cisac_id.clone().into()),
            ("@dpid", self.dpid.clone().into()),
            ("@ipi", self.ipi.into()),
            ("@contact_name", self.contact_name.clone().into()),
            ("@full_name", self.full_name.clone().into()),
            ("@key_name", self.last_name.clone().into()),
            ("@prefix", self.first_name.clone().into()),
        ])
    }

    fn migrate(conn: &sqlite::Connection) -> Result<(), sqlite::Error> {
        let sql = "
        CREATE TABLE IF NOT EXISTS parties (
           id INTEGER PRIMARY KEY NOT NULL,
           email TEXT,
           isni TEXT,
           cisac_id TEXT,
           dpid TEXT,
           ipi INTEGER,
           contact_name TEXT,
           full_name TEXT NOT NULL,
           key_name TEXT,
           prefix TEXT
        )";
        conn.execute(sql)
    }
}
