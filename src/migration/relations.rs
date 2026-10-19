use rusqlite::params;

pub struct PublisherRelations {
    // pub parent_id: i64,
    // pub child_id: i64,
}

impl PublisherRelations {
    pub fn migrate(conn: &rusqlite::Connection) -> Result<(), rusqlite::Error> {
        let sql = "
        CREATE TABLE IF NOT EXISTS publisher_relations (
            parent_id INTEGER NOT NULL REFERENCES parties(id) ON DELETE CASCADE,
            child_id INTEGER NOT NULL REFERENCES parties(id) ON DELETE CASCADE,
            occurrences INTEGER NOT NULL DEFAULT 1,
            PRIMARY KEY(parent_id, child_id)
        )";
        _ = conn.execute(sql, params!())?;
        Ok(())
    }
}

pub struct WriterRelations {
    // pub writer_a: i64,
    // pub writer_b: i64,
}

impl WriterRelations {
    pub fn migrate(conn: &rusqlite::Connection) -> Result<(), rusqlite::Error> {
        let sql = "
        CREATE TABLE IF NOT EXISTS writer_relations (
            writer_a INTEGER NOT NULL REFERENCES parties(id) ON DELETE CASCADE,
            writer_b INTEGER NOT NULL REFERENCES parties(id) ON DELETE CASCADE,
            occurrences INTEGER NOT NULL DEFAULT 1,
            PRIMARY KEY(writer_a, writer_b)
        )";
        _ = conn.execute(sql, params!())?;
        Ok(())
    }
}
