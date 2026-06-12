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

async fn modify_parties_migration(conn: &libsql::Connection) -> Result<(), libsql::Error> {
    let sql = "
        ALTER TABLE parties
        ADD COLUMN pro INTEGER;
        ALTER TABLE parties
        ADD COLUMN mro INTEGER;
        ALTER TABLE parties
        ADD COLUMN sro INTEGER;
        ALTER TABLE parties
        ADD COLUMN role TEXT;
        ";
    _ = conn.execute(sql, params!()).await?;
    Ok(())
}

pub async fn migrate_add_ons(conn: &libsql::Connection) -> Result<(), libsql::Error> {
    // modify_parties_migration(conn)?;
    PublisherRelations::migrate(conn).await?;
    WriterRelations::migrate(conn).await?;
    index::create_indexes(conn).await?;
    Ok(())
}
