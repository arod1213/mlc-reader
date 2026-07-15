use crate::{
    bwarm::{
        party::Party, release::Release, resource::Resource, share::Share,
        work_resource::WorkResource, works::Work,
    },
    migration::utils::{disable_fk, migrate_schema, save_object},
    server::{self, Credential},
};
use libsql::{Connection, params};
use std::path::Path;

pub mod utils;

/// save TSV files from SFTP
pub fn save_remote_mlc_docs(cred: &Credential) {
    let sftp = cred.open().unwrap();
    let dir = server::latest_dir(&sftp).expect("missing MLC dirs");
    server::save_doc::<Resource>(&sftp, &dir).expect("failed to save Resources");
    server::save_doc::<Release>(&sftp, &dir).expect("failed to save Releases");
    server::save_doc::<Work>(&sftp, &dir).expect("failed to save Works");
    server::save_doc::<WorkResource>(&sftp, &dir).expect("failed to save WorkResources");
    server::save_doc::<Party>(&sftp, &dir).expect("failed to save Parties");
    server::save_doc::<Share>(&sftp, &dir).expect("failed to save Shares");
}

/// migrate DB and save TSV files into db
pub async fn migrate_from_bwarm_dump(conn: &Connection, bwarm_dir: &Path) {
    migrate_schema(conn).await.expect("failed to migrate");

    disable_fk(conn).await.expect("failed to disable FKs");

    let tx = conn
        .transaction()
        .await
        .expect("failed to setup transaction");

    let res = async {
        save_object::<Release>(&tx, bwarm_dir).await?;
        save_object::<Resource>(&tx, bwarm_dir).await?;
        save_object::<Party>(&tx, bwarm_dir).await?;
        save_object::<Work>(&tx, bwarm_dir).await?;
        save_object::<WorkResource>(&tx, bwarm_dir).await?;
        save_object::<Share>(&tx, bwarm_dir).await?;
        Ok::<_, Box<dyn std::error::Error>>(())
    }
    .await;
    match res {
        Ok(_) => {
            tx.commit().await.expect("failed to commit");
            println!("inserted BWARM");
        }
        Err(e) => {
            tx.rollback().await.expect("failed to rollback");
            println!("failed: {}", e);
        }
    }
}

/// add new tables and indexes
pub async fn migrate_add_ons(conn: &libsql::Connection) -> Result<(), libsql::Error> {
    // modify_parties_migration(conn)?;
    PublisherRelations::migrate(conn).await?;
    WriterRelations::migrate(conn).await?;
    create_indexes(conn).await?;
    create_party_fts(conn).await?;
    Ok(())
}

pub async fn create_indexes(conn: &Connection) -> Result<(), libsql::Error> {
    create_publisher_relation_index(conn).await?;
    create_share_index(conn).await?;
    create_relation_index(conn).await?;
    create_work_resource_index(conn).await?;
    Ok(())
}

/// indicate whether a party is primarily a writer / publisher
pub async fn assign_roles(conn: &libsql::Transaction) -> Result<(), libsql::Error> {
    let sql = "
    WITH role_counts AS (
        SELECT
          id,
          COALESCE(writer_count, 0) AS writer_count,
          COALESCE(publisher_count, 0) AS publisher_count
        FROM parties
        LEFT JOIN (
          SELECT child_id AS id, SUM(occurrences) AS writer_count
          FROM publisher_relations
          GROUP BY child_id
    ) w USING (id)
    LEFT JOIN (
      SELECT parent_id AS id, SUM(occurrences) AS publisher_count
      FROM publisher_relations
      GROUP BY parent_id
    ) p USING (id)
    )
    UPDATE parties
    SET role = (
    SELECT CASE
      WHEN writer_count > publisher_count THEN 'writer'
      WHEN writer_count < publisher_count THEN 'publisher'
      WHEN writer_count = publisher_count AND writer_count > 0 THEN 'both'
      ELSE NULL
    END
    FROM role_counts
    WHERE role_counts.id = parties.id
    );";
    _ = conn.execute(sql, params!()).await?;
    Ok(())
}

/// save writer -> publisher and publisher -> publisher instances
pub async fn enrich_publisher_relations(conn: &libsql::Connection) -> Result<(), libsql::Error> {
    let sql = "
        WITH relations AS (
            SELECT
                p.id  AS parent_id,
                rp.id AS child_id
            FROM parties p
            JOIN shares s  ON s.party_id = p.id
            JOIN shares rs ON rs.preceding_id = s.id
            JOIN parties rp ON rp.id = rs.party_id
        ),
        relation_counts AS (
            SELECT parent_id, child_id, COUNT(*) occurrences
            FROM relations
            GROUP BY parent_id, child_id
        )
        INSERT INTO publisher_relations (parent_id, child_id, occurrences)
        SELECT parent_id, child_id, occurrences
        FROM relation_counts
        WHERE true
        ON CONFLICT(parent_id, child_id) DO UPDATE
        SET occurrences = occurrences + excluded.occurrences;
        ";
    _ = conn.execute(sql, params!()).await?;
    Ok(())
}

/// save writer -> writer instances
pub async fn enrich_writer_relations(conn: &libsql::Transaction) -> Result<(), libsql::Error> {
    let sql = "
          WITH root_writer_shares AS (
              SELECT DISTINCT
                  s.work_id,
                  s.party_id
              FROM shares s
              WHERE NOT EXISTS (
                  SELECT 1
                  FROM shares rs
                  WHERE rs.work_id = s.work_id
                    AND rs.preceding_id = s.id
              )
          ),
          pairs AS (
            SELECT a.party_id writer_a, b.party_id writer_b
            FROM root_writer_shares a
            JOIN root_writer_shares b
              ON a.work_id = b.work_id
             AND a.party_id < b.party_id
          ),
          pair_counts AS (
            SELECT writer_a, writer_b, COUNT(*) occurrences
            FROM pairs
            GROUP BY writer_a, writer_b
          )
          INSERT INTO writer_relations (writer_a, writer_b, occurrences)
          SELECT writer_a, writer_b, occurrences
          FROM pair_counts
          WHERE true
          ON CONFLICT(writer_a, writer_b) DO UPDATE
          SET occurrences = occurrences + excluded.occurrences;
        ";
    _ = conn.execute(sql, params!()).await?;
    Ok(())
}

// ---------------------
// TABLES

struct PublisherRelations {
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

struct WriterRelations {
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

//---------------------------
// INDEXES

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

async fn create_work_resource_index(conn: &Connection) -> Result<(), libsql::Error> {
    let sql = "
        CREATE INDEX IF NOT EXISTS idx_work_resources_work_resource
        ON work_resources(work_id, resource_id);
        ";
    _ = conn.execute(sql, params!()).await?;
    Ok(())
}

async fn create_relation_index(conn: &Connection) -> Result<(), libsql::Error> {
    let sql = "
    CREATE INDEX IF NOT EXISTS idx_publisher_relations_child_occ
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

        CREATE INDEX idx_shares_work_preceeding_shares
        ON shares(work_id, preceding_id);

        CREATE INDEX IF NOT EXISTS idx_shares_work_party
        ON shares(work_id, party_id);
        ";
    _ = conn.execute(sql, params!()).await?;
    Ok(())
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
