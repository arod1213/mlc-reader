use crate::{
    bwarm::{
        party::Party, release::Release, resource::Resource, share::Share,
        work_resource::WorkResource, works::Work,
    },
    migration::{
        trim::setup_bulk_write_mode,
        utils::{disable_fk, migrate_schema, save_object},
    },
    server::{self, Credential},
};
use libsql::{Connection, params};
use std::path::{Path, PathBuf};

mod trim;
pub mod utils;

/// save TSV files from SFTP
pub fn save_remote_mlc_docs(cred: &Credential, out_dir: &PathBuf) {
    let sftp = cred.open().unwrap();
    let dir = server::latest_dir(&sftp).expect("missing MLC dirs");
    println!("dir is {:?}", dir);

    // TODO: make multithreaded (too slow rn)
    server::save_doc::<Party>(&sftp, &dir, &out_dir).expect("failed to save Parties");
    server::save_doc::<Work>(&sftp, &dir, &out_dir).expect("failed to save Works");
    server::save_doc::<WorkResource>(&sftp, &dir, &out_dir).expect("failed to save WorkResources");
    server::save_doc::<Share>(&sftp, &dir, &out_dir).expect("failed to save Shares");
    server::save_doc::<Release>(&sftp, &dir, &out_dir).expect("failed to save Releases");
    server::save_doc::<Resource>(&sftp, &dir, &out_dir).expect("failed to save Resources");
}

pub async fn trim_db(conn: &Connection, vacuum: bool) -> Result<(), libsql::Error> {
    trim::setup_bulk_write_mode(conn).await?;

    create_trim_shares_indexes(conn).await?;
    trim::trim_shares(conn).await?;
    println!("trimmed shares");

    create_trim_works_indexes(conn).await?;
    trim::trim_works(conn).await?;
    println!("trimmed works");

    create_trim_releases_indexes(conn).await?;
    trim::trim_releases(conn).await?;
    println!("trimmed releases");

    create_trim_parties_indexes(conn).await?;
    trim::trim_parties(conn).await?;
    println!("trimmed parties");

    if vacuum {
        _ = conn.execute("VACUUM", params!()).await?;
    }
    Ok(())
}

/// migrate DB and save TSV files into db
pub async fn migrate_from_bwarm_dump(conn: &Connection, bwarm_dir: &Path) {
    migrate_schema(conn).await.expect("failed to migrate");

    setup_bulk_write_mode(conn)
        .await
        .expect("failed to setup WAL");
    disable_fk(conn).await.expect("failed to disable FKs");

    save_object::<Release>(conn, bwarm_dir).await.unwrap();
    save_object::<Resource>(conn, bwarm_dir).await.unwrap();
    save_object::<Party>(conn, bwarm_dir).await.unwrap();
    save_object::<Work>(conn, bwarm_dir).await.unwrap();
    save_object::<WorkResource>(conn, bwarm_dir).await.unwrap();
    save_object::<Share>(conn, bwarm_dir).await.unwrap();
}

/// add new tables and indexes
pub async fn create_search_tables_indexes(conn: &libsql::Connection) -> Result<(), libsql::Error> {
    // modify_parties_migration(conn)?;
    PublisherRelations::migrate(conn).await?;
    WriterRelations::migrate(conn).await?;
    create_search_indexes(conn).await?;
    create_party_fts(conn).await?;
    create_relation_index(conn).await?;
    Ok(())
}

pub async fn create_search_indexes(conn: &Connection) -> Result<(), libsql::Error> {
    create_publisher_relation_index(conn).await?;
    create_party_indexes(conn).await?;
    create_share_index(conn).await?;
    create_relation_index(conn).await?;
    create_work_indexes(conn).await?;
    Ok(())
}

pub async fn create_trim_indexes(conn: &Connection) -> Result<(), libsql::Error> {
    create_trim_shares_indexes(conn).await?;
    create_trim_works_indexes(conn).await?;
    create_trim_releases_indexes(conn).await?;
    create_trim_parties_indexes(conn).await?;
    Ok(())
}

pub async fn create_trim_shares_indexes(conn: &Connection) -> Result<(), libsql::Error> {
    _ = conn
        .execute(
            "CREATE INDEX IF NOT EXISTS idx_shares_preceding_id ON shares(preceding_id);",
            params!(),
        )
        .await?;
    Ok(())
}

pub async fn create_trim_works_indexes(conn: &Connection) -> Result<(), libsql::Error> {
    _ = conn
        .execute(
            "CREATE INDEX IF NOT EXISTS idx_shares_work_party ON shares(work_id, party_id);",
            params!(),
        )
        .await?;
    _ = conn
        .execute(
            "CREATE INDEX IF NOT EXISTS idx_work_resources_work_resource ON work_resources(work_id, resource_id);",
            params!(),
        )
        .await?;
    Ok(())
}

pub async fn create_trim_releases_indexes(conn: &Connection) -> Result<(), libsql::Error> {
    _ = conn
        .execute(
            "CREATE INDEX IF NOT EXISTS idx_resources_release_id_id ON resources(release_id, id);",
            params!(),
        )
        .await?;
    _ = conn
        .execute(
            "CREATE INDEX IF NOT EXISTS idx_work_resources_resource_id ON work_resources(resource_id);",
            params!(),
        )
        .await?;
    Ok(())
}

pub async fn create_trim_parties_indexes(conn: &Connection) -> Result<(), libsql::Error> {
    _ = conn
        .execute(
            "CREATE INDEX IF NOT EXISTS idx_party_ipi ON parties(ipi);",
            params!(),
        )
        .await?;
    _ = conn
        .execute(
            "CREATE INDEX IF NOT EXISTS idx_shares_party_id ON shares(party_id);",
            params!(),
        )
        .await?;
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

/// indicate whether a party is primarily a writer / publisher
pub async fn add_party_stats(conn: &libsql::Transaction) -> Result<(), libsql::Error> {
    conn.execute(
        "ALTER TABLE parties ADD COLUMN average_share REAL NOT NULL DEFAULT 0.0;",
        params!(),
    )
    .await?;

    conn.execute(
        "ALTER TABLE parties ADD COLUMN latest_release INTEGER",
        params!(),
    )
    .await?;

    conn.execute(
        "ALTER TABLE parties ADD COLUMN work_count INTEGER NOT NULL DEFAULT 0",
        params!(),
    )
    .await?;

    let sql = "
        WITH party_stats AS (
          SELECT
              s.party_id,
              COALESCE(AVG(s.share), 0) AS average_share,
              COUNT(DISTINCT s.work_id) AS work_count,
              MAX(r.release_date) AS latest_release
          FROM shares AS s
          LEFT JOIN work_resources wr on wr.work_id = s.work_id
          LEFT JOIN resources rs on rs.id = wr.resource_id
          LEFT JOIN releases r on r.id = rs.release_id
          GROUP BY s.party_id
        )
        UPDATE parties AS p
        SET
          latest_release = (SELECT ps.latest_release FROM party_stats ps WHERE ps.party_id = p.id),
          average_share = COALESCE(
              (SELECT ps.average_share FROM party_stats ps WHERE ps.party_id = p.id),
              0
          ),
          work_count = COALESCE(
              (SELECT ps.work_count FROM party_stats ps WHERE ps.party_id = p.id),
              0
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

async fn create_party_indexes(conn: &Connection) -> Result<(), libsql::Error> {
    let sql = "
        CREATE INDEX IF NOT EXISTS idx_party_ipi ON parties(ipi);
        ";
    _ = conn.execute(sql, params!()).await?;
    Ok(())
}

async fn create_publisher_relation_index(conn: &Connection) -> Result<(), libsql::Error> {
    let sql = "
        CREATE INDEX IF NOT EXISTS idx_publisher_relations_parent_occ
        ON publisher_relations (parent_id, occurrences DESC);
        ";
    _ = conn.execute(sql, params!()).await?;
    let sql = "
        CREATE INDEX IF NOT EXISTS idx_publisher_relations_child_occ
        ON publisher_relations (child_id, occurrences DESC);
        ";
    _ = conn.execute(sql, params!()).await?;
    Ok(())
}

async fn create_work_indexes(conn: &Connection) -> Result<(), libsql::Error> {
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
    _ = conn
        .execute(
            "CREATE INDEX IF NOT EXISTS idx_shares_party_id ON shares(party_id);",
            params!(),
        )
        .await?;
    _ = conn
        .execute(
            "CREATE INDEX IF NOT EXISTS idx_shares_preceding_id ON shares(preceding_id);",
            params!(),
        )
        .await?;
    _ = conn
        .execute(
            "CREATE INDEX IF NOT EXISTS idx_shares_preceding_party ON shares(preceding_id, party_id);",
            params!(),
        )
        .await?;
    _ = conn
        .execute(
            "CREATE INDEX IF NOT EXISTS idx_shares_work_preceeding_shares ON shares(work_id, preceding_id);",
            params!(),
        )
        .await?;
    _ = conn
        .execute(
            "CREATE INDEX IF NOT EXISTS idx_shares_work_party ON shares(work_id, party_id);",
            params!(),
        )
        .await?;

    Ok(())
}

pub async fn create_resource_idx(conn: &libsql::Connection) -> Result<(), libsql::Error> {
    let sql = "CREATE UNIQUE INDEX resources_id_idx ON resources_staging(id);";
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
