use libsql::params;

pub mod create;
mod index;
pub mod local;
pub mod search;

pub struct PublisherRelations {
    // pub parent_id: i64,
    // pub child_id: i64,
}

impl PublisherRelations {
    pub async fn migrate(conn: &libsql::Connection) -> Result<(), libsql::Error> {
        let sql = "
        CREATE TABLE IF NOT EXISTS publisher_relations (
            parent_id INTEGER NOT NULL REFERENCES parties(id) ON DELETE CASCADE,
            child_id INTEGER NOT NULL REFERENCES parties(id) ON DELETE CASCADE,
            occurrences INTEGER NOT NULL DEFAULT 1,
            PRIMARY KEY(parent_id, child_id)
        )";
        _ = conn.execute(sql, params!()).await?;
        Ok(())
    }
}

pub struct WriterRelations {
    // pub writer_a: i64,
    // pub writer_b: i64,
}

impl WriterRelations {
    pub async fn migrate(conn: &libsql::Connection) -> Result<(), libsql::Error> {
        let sql = "
        CREATE TABLE IF NOT EXISTS writer_relations (
            writer_a INTEGER NOT NULL REFERENCES parties(id) ON DELETE CASCADE,
            writer_b INTEGER NOT NULL REFERENCES parties(id) ON DELETE CASCADE,
            occurrences INTEGER NOT NULL DEFAULT 1,
            PRIMARY KEY(writer_a, writer_b)
        )";
        _ = conn.execute(sql, params!()).await?;
        Ok(())
    }
}

pub async fn create_party_fts(conn: &libsql::Connection) -> Result<(), libsql::Error> {
    let sql = "
        CREATE VIRTUAL TABLE IF NOT EXISTS parties_fts USING fts5(
            full_name,
            content=parties,
            content_rowid=id,
            tokenize='unicode61'
        );";
    _ = conn.execute(sql, params!()).await?;

    let sql = "
        -- Populate from existing data
        INSERT INTO parties_fts(rowid, full_name)
        SELECT id, full_name FROM parties;";
    _ = conn.execute(sql, params!()).await?;

    let sql = "
        CREATE TRIGGER IF NOT EXISTS parties_fts_insert AFTER INSERT ON parties BEGIN
            INSERT INTO parties_fts(rowid, full_name) VALUES (new.id, new.full_name);
        END;";
    _ = conn.execute(sql, params!()).await?;

    let sql = "
        CREATE TRIGGER IF NOT EXISTS parties_fts_delete AFTER DELETE ON parties BEGIN
            INSERT INTO parties_fts(parties_fts, rowid, full_name) VALUES ('delete', old.id, old.full_name);
        END;";
    _ = conn.execute(sql, params!()).await?;

    let sql = "
        CREATE TRIGGER IF NOT EXISTS parties_fts_update
        AFTER UPDATE OF full_name ON parties
        BEGIN
          INSERT INTO parties_fts(parties_fts, rowid, full_name)
          VALUES ('delete', old.id, old.full_name);

          INSERT INTO parties_fts(rowid, full_name)
          VALUES (new.id, new.full_name);
        END;";
    _ = conn.execute(sql, params!()).await?;
    Ok(())
}

pub async fn migrate_add_ons(conn: &libsql::Connection) -> Result<(), libsql::Error> {
    // modify_parties_migration(conn)?;
    PublisherRelations::migrate(conn).await?;
    WriterRelations::migrate(conn).await?;
    index::create_indexes(conn).await?;
    create_party_fts(conn).await?;
    Ok(())
}
