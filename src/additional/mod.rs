pub mod create;
mod index;
pub mod local;

pub struct PublisherRelations {
    pub parent_id: i64,
    pub child_id: i64,
}

impl PublisherRelations {
    pub fn migrate(conn: &sqlite::Connection) -> Result<(), sqlite::Error> {
        let sql = "
        CREATE TABLE IF NOT EXISTS publisher_relations (
            parent_id INTEGER NOT NULL REFERENCES parties(id) ON DELETE CASCADE,
            child_id INTEGER NOT NULL REFERENCES parties(id) ON DELETE CASCADE,
            occurrences INTEGER NOT NULL DEFAULT 1,
            PRIMARY KEY(parent_id, child_id)
        )";
        conn.execute(sql)
    }
}

pub struct WriterRelations {
    pub writer_a: i64,
    pub writer_b: i64,
}

impl WriterRelations {
    pub fn migrate(conn: &sqlite::Connection) -> Result<(), sqlite::Error> {
        let sql = "
        CREATE TABLE IF NOT EXISTS writer_relations (
            writer_a INTEGER NOT NULL REFERENCES parties(id) ON DELETE CASCADE,
            writer_b INTEGER NOT NULL REFERENCES parties(id) ON DELETE CASCADE,
            occurrences INTEGER NOT NULL DEFAULT 1,
            PRIMARY KEY(writer_a, writer_b)
        )";
        conn.execute(sql)
    }
}

fn modify_parties_migration(conn: &sqlite::Connection) -> Result<(), sqlite::Error> {
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
    conn.execute(sql)
}

pub fn migrate_add_ons(conn: &sqlite::Connection) -> Result<(), sqlite::Error> {
    // modify_parties_migration(conn)?;
    PublisherRelations::migrate(conn)?;
    WriterRelations::migrate(conn)?;
    index::create_indexes(conn)?;
    Ok(())
}
