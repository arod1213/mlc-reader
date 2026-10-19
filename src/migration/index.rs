use crate::migration::{
    pks::{create_party_pk, create_resource_pk, create_share_pk, create_work_pk},
    relations::{PublisherRelations, WriterRelations},
    setup,
};
use rusqlite::{Connection, params};

pub fn setup_indexes(conn: &Connection) -> Result<(), rusqlite::Error> {
    setup::setup_trim_write_mode(conn)?;

    log::info!("creating shares idx");
    create_trim_shares_indexes(conn)?;
    log::info!("creating resource pk");
    create_share_pk(conn)?;

    log::info!("creating resource pk");
    create_resource_pk(conn)?;
    log::info!("creating trim_works idx");
    create_trim_works_indexes(conn)?;
    log::info!("creating works pk");
    create_work_pk(conn)?;

    log::info!("creating releases idx");
    create_trim_releases_indexes(conn)?;

    log::info!("creating parties pk");
    create_party_pk(conn)?;
    log::info!("creating parties idx");
    create_trim_parties_indexes(conn)?;

    Ok(())
}

/// add new tables and indexes
pub fn create_relationship_indexes(conn: &rusqlite::Connection) -> Result<(), rusqlite::Error> {
    // modify_parties_migration(conn)?;
    log::info!("creating publisher relations");
    PublisherRelations::migrate(conn)?;
    log::info!("creating writer relations");
    WriterRelations::migrate(conn)?;
    log::info!("creating search index");
    create_search_indexes(conn)?;
    log::info!("creating party fts");
    create_party_fts(conn)?;
    log::info!("creating relation index");
    create_relation_index(conn)?;
    log::info!("finished indexes");
    Ok(())
}

pub fn create_search_indexes(conn: &Connection) -> Result<(), rusqlite::Error> {
    log::info!("creating publisher relation index");
    create_publisher_relation_index(conn)?;
    log::info!("creating party index");
    create_party_indexes(conn)?;
    log::info!("creating share index");
    create_share_index(conn)?;
    log::info!("creating relation index");
    create_relation_index(conn)?;
    log::info!("creating work index");
    create_work_indexes(conn)?;
    Ok(())
}

pub fn create_trim_indexes(conn: &Connection) -> Result<(), rusqlite::Error> {
    create_trim_shares_indexes(conn)?;
    create_trim_works_indexes(conn)?;
    create_trim_releases_indexes(conn)?;
    create_trim_parties_indexes(conn)?;

    Ok(())
}

pub fn create_trim_shares_indexes(conn: &Connection) -> Result<(), rusqlite::Error> {
    _ = conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_shares_preceding_id ON shares(preceding_id);",
        params!(),
    )?;
    Ok(())
}

pub fn create_trim_works_indexes(conn: &Connection) -> Result<(), rusqlite::Error> {
    _ = conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_shares_work_party ON shares(work_id, party_id);",
        params!(),
    )?;
    _ = conn
        .execute(
            "CREATE INDEX IF NOT EXISTS idx_work_resources_work_resource ON work_resources(work_id, resource_id);",
            params!(),
        )
        ?;
    _ = conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_resources_isrc_id ON resources(isrc, id);",
        params!(),
    )?;
    Ok(())
}

pub fn create_trim_releases_indexes(conn: &Connection) -> Result<(), rusqlite::Error> {
    _ = conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_resources_release_id_id ON resources(release_id, id);",
        params!(),
    )?;
    _ = conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_work_resources_resource_id ON work_resources(resource_id);",
        params!(),
    )?;
    Ok(())
}

pub fn create_trim_parties_indexes(conn: &Connection) -> Result<(), rusqlite::Error> {
    _ = conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_party_ipi ON parties(ipi);",
        params!(),
    )?;
    _ = conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_shares_party_id ON shares(party_id);",
        params!(),
    )?;
    Ok(())
}

fn create_party_indexes(conn: &Connection) -> Result<(), rusqlite::Error> {
    let sql = "
        CREATE INDEX IF NOT EXISTS idx_party_ipi ON parties(ipi);
        ";
    _ = conn.execute(sql, params!())?;
    Ok(())
}

fn create_publisher_relation_index(conn: &Connection) -> Result<(), rusqlite::Error> {
    let sql = "
        CREATE INDEX IF NOT EXISTS idx_publisher_relations_parent_occ
        ON publisher_relations (parent_id, occurrences DESC);
        ";
    _ = conn.execute(sql, params!())?;
    let sql = "
        CREATE INDEX IF NOT EXISTS idx_publisher_relations_child_occ
        ON publisher_relations (child_id, occurrences DESC);
        ";
    _ = conn.execute(sql, params!())?;
    Ok(())
}

fn create_work_indexes(conn: &Connection) -> Result<(), rusqlite::Error> {
    let sql = "
        CREATE INDEX IF NOT EXISTS idx_work_resources_work_resource
        ON work_resources(work_id, resource_id);
        ";
    _ = conn.execute(sql, params!())?;
    Ok(())
}

fn create_relation_index(conn: &Connection) -> Result<(), rusqlite::Error> {
    let sql = "
    CREATE INDEX IF NOT EXISTS idx_publisher_relations_child_occ
    ON publisher_relations (child_id, occurrences DESC);
        ";
    _ = conn.execute(sql, params!())?;
    Ok(())
}

fn create_share_index(conn: &Connection) -> Result<(), rusqlite::Error> {
    _ = conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_shares_party_id ON shares(party_id);",
        params!(),
    )?;
    _ = conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_shares_preceding_id ON shares(preceding_id);",
        params!(),
    )?;
    _ = conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_shares_preceding_party ON shares(preceding_id, party_id);",
        params!(),
    )?;
    _ = conn
        .execute(
            "CREATE INDEX IF NOT EXISTS idx_shares_work_preceeding_shares ON shares(work_id, preceding_id);",
            params!(),
        )
        ?;
    _ = conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_shares_work_party ON shares(work_id, party_id);",
        params!(),
    )?;

    Ok(())
}

pub fn create_party_fts(conn: &rusqlite::Connection) -> Result<(), rusqlite::Error> {
    let sql = "
        CREATE VIRTUAL TABLE IF NOT EXISTS parties_fts USING fts5(
            full_name,
            content=parties,
            content_rowid=id,
            tokenize='unicode61'
        );";
    _ = conn.execute(sql, params!())?;

    let sql = "
        -- Populate from existing data
        INSERT INTO parties_fts(rowid, full_name)
        SELECT id, full_name FROM parties;";
    _ = conn.execute(sql, params!())?;

    let sql = "
        CREATE TRIGGER IF NOT EXISTS parties_fts_insert AFTER INSERT ON parties BEGIN
            INSERT INTO parties_fts(rowid, full_name) VALUES (new.id, new.full_name);
        END;";
    _ = conn.execute(sql, params!())?;

    let sql = "
        CREATE TRIGGER IF NOT EXISTS parties_fts_delete AFTER DELETE ON parties BEGIN
            INSERT INTO parties_fts(parties_fts, rowid, full_name) VALUES ('delete', old.id, old.full_name);
        END;";
    _ = conn.execute(sql, params!())?;

    let sql = "
        CREATE TRIGGER IF NOT EXISTS parties_fts_update
        AFTER UPDATE OF full_name ON parties
        BEGIN
          INSERT INTO parties_fts(parties_fts, rowid, full_name)
          VALUES ('delete', old.id, old.full_name);

          INSERT INTO parties_fts(rowid, full_name)
          VALUES (new.id, new.full_name);
        END;";
    _ = conn.execute(sql, params!())?;
    Ok(())
}
