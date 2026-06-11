use libsql::{Connection, params};

pub async fn create_indexes(conn: &Connection) -> Result<(), libsql::Error> {
    create_publisher_relation_index(conn).await?;
    create_share_index(conn).await?;
    Ok(())
}

async fn create_publisher_relation_index(conn: &Connection) -> Result<(), libsql::Error> {
    let sql = "
        CREATE INDEX idx_publisher_relations_parent_occ
        ON publisher_relations (parent_id, occurrences DESC);
        CREATE INDEX idx_publisher_relations_child_occ
        ON publisher_relations (child_id, occurrences DESC);
        ";
    _ = conn.execute(sql, params!()).await?;
    Ok(())
}
async fn create_share_index(conn: &Connection) -> Result<(), libsql::Error> {
    let sql = "
        CREATE INDEX idx_shares_party_id
        ON shares(party_id);

        CREATE INDEX idx_shares_preceding_id
        ON shares(preceding_id);

        CREATE INDEX idx_shares_preceding_party
        ON shares(preceding_id, party_id);
        ";
    _ = conn.execute(sql, params!()).await?;
    Ok(())
}

// CREATE TRIGGER parties_fts_insert AFTER INSERT ON parties BEGIN
// INSERT INTO parties_fts(rowid, full_name) VALUES (new.id, new.full_name);
// END;
// CREATE TRIGGER parties_fts_delete AFTER DELETE ON parties BEGIN
// INSERT INTO parties_fts(parties_fts, rowid, full_name) VALUES ('delete', old.id, old.full_name);
// END;
// CREATE TRIGGER parties_fts_update AFTER UPDATE ON parties BEGIN
// INSERT INTO parties_fts(parties_fts, rowid, full_name) VALUES ('delete', old.id, old.full_name);
// INSERT INTO parties_fts(rowid, full_name) VALUES (new.id, new.full_name);
// END;
// CREATE INDEX idx_parties_ipi
// ON parties (ipi);
// CREATE VIRTUAL TABLE parties_fts USING fts5(
// full_name,
// content=parties,
// content_rowid=id,
// tokenize='unicode61'
// );
// CREATE TABLE 'parties_fts_config'(k PRIMARY KEY, v) WITHOUT ROWID;
// CREATE TABLE 'parties_fts_data'(id INTEGER PRIMARY KEY, block BLOB);
// CREATE TABLE 'parties_fts_docsize'(id INTEGER PRIMARY KEY, sz BLOB);
// CREATE TABLE 'parties_fts_idx'(segid, term, pgno, PRIMARY KEY(segid, term)) WITHOUT ROWID;
// CREATE TABLE writer_relations (
// parent_id INTEGER NOT NULL,
// child_id INTEGER NOT NULL,
// occurrences INTEGER NOT NULL DEFAULT 1,
// UNIQUE(parent_id, child_id),
// FOREIGN KEY (parent_id) REFERENCES parties(id) ON DELETE CASCADE,
// FOREIGN KEY (child_id) REFERENCES parties(id) ON DELETE CASCADE
// );
// CREATE INDEX idx_writer_relations_parent_occ
// ON writer_relations (parent_id, occurrences DESC);
// CREATE INDEX idx_writer_relations_child_occ
// ON writer_relations (child_id, occurrences DESC);
