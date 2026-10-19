use crate::{
    bwarm::{
        party::Party, release::Release, resource::Resource, share::Share,
        work_resource::WorkResource, works::Work,
    },
    migration::{
        setup::{disable_fk, setup_bulk_write_mode},
        utils::{migrate_schema, save_object},
    },
    server::{self, Credential},
};
use rusqlite::{Connection, params};
use std::{error::Error, path::Path};

pub mod index;
mod pks;
pub mod relations;
pub mod setup;
pub mod utils;

pub fn migrate_full_db(conn: &rusqlite::Connection) -> Result<(), rusqlite::Error> {
    migrate_schema(conn)?;
    index::create_relationship_indexes(conn)?;
    migrate_party_stats_fields(conn)?;
    migrate_work_totals_fields(conn)?;
    Ok(())
}

/// save TSV files from SFTP
pub fn save_mlc_tsv_files(cred: &Credential, out_dir: &Path) -> Result<(), Box<dyn Error>> {
    let sftp = cred.open().unwrap();
    let Some(dir) = server::latest_dir(&sftp) else {
        return Err("Missing MLC dirs".into());
    };
    log::debug!("dir is {:?}", dir);

    // TODO: make multithreaded (too slow rn)
    server::save_doc::<Party>(&sftp, &dir, out_dir).expect("missing parties file");
    server::save_doc::<Work>(&sftp, &dir, out_dir).expect("missing work file");
    server::save_doc::<WorkResource>(&sftp, &dir, out_dir).expect("missing work resource file");
    server::save_doc::<Share>(&sftp, &dir, out_dir).expect("missing share file");
    server::save_doc::<Release>(&sftp, &dir, out_dir).expect("missing release file");
    server::save_doc::<Resource>(&sftp, &dir, out_dir).expect("missing resource file");
    Ok(())
}

/// migrate DB and save TSV files into db
pub fn insert_tsv_files_to_db(
    conn: &mut Connection,
    bwarm_dir: &Path,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    migrate_schema(conn)?;

    setup_bulk_write_mode(conn)?;
    disable_fk(conn)?;

    save_object::<Release>(conn, bwarm_dir)?;
    save_object::<Resource>(conn, bwarm_dir)?;
    save_object::<Party>(conn, bwarm_dir)?;
    save_object::<Work>(conn, bwarm_dir)?;
    save_object::<WorkResource>(conn, bwarm_dir)?;
    save_object::<Share>(conn, bwarm_dir)?;
    Ok(())
}

/// indicate whether a party is primarily a writer / publisher
pub fn assign_roles(conn: &rusqlite::Transaction) -> Result<(), rusqlite::Error> {
    let sql = "
        WITH counts AS (
          SELECT
            party_id,
            SUM(role IN ('ComposerLyricist','Composer','Lyricist','Arranger',
                         'SubLyricist','Adapter','Translator','SubArranger')) AS wc,
            SUM(role IN ('OriginalPublisher','RightsAdministrator','SubPublisher',
                         'SubstitutedPublisher')) AS pc
          FROM shares NOT INDEXED
          GROUP BY party_id
        )
        UPDATE parties SET role = CASE
            WHEN c.wc > c.pc THEN 'writer'
            WHEN c.wc < c.pc THEN 'publisher'
            WHEN c.wc > 0    THEN 'both'
          END
        FROM counts c
        WHERE c.party_id = parties.id
    ";
    _ = conn.execute(sql, params!())?;
    Ok(())
}

fn migrate_party_stats_fields(conn: &rusqlite::Connection) -> Result<(), rusqlite::Error> {
    conn.execute(
        "ALTER TABLE parties ADD COLUMN average_share REAL NOT NULL DEFAULT 0.0;",
        params!(),
    )?;

    conn.execute(
        "ALTER TABLE parties ADD COLUMN latest_release INTEGER",
        params!(),
    )?;

    conn.execute(
        "ALTER TABLE parties ADD COLUMN work_count INTEGER NOT NULL DEFAULT 0",
        params!(),
    )?;
    Ok(())
}

/// indicate whether a party is primarily a writer / publisher
pub fn add_party_stats(conn: &rusqlite::Transaction) -> Result<(), rusqlite::Error> {
    migrate_party_stats_fields(conn)?;
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

    _ = conn.execute(sql, params!())?;
    Ok(())
}

fn migrate_work_totals_fields(conn: &rusqlite::Connection) -> Result<(), rusqlite::Error> {
    _ = conn.execute(
        "ALTER TABLE works ADD COLUMN total_shares REAL NOT NULL DEFAULT 0;",
        [],
    )?;
    Ok(())
}

/// save writer -> publisher and publisher -> publisher instances
pub fn enrich_work_totals(conn: &rusqlite::Connection) -> Result<(), rusqlite::Error> {
    migrate_work_totals_fields(conn)?;

    let sql = "
        WITH totals AS (
            SELECT work_id, SUM(share) AS share
            FROM shares
            GROUP BY work_id
        )
        UPDATE works
        SET total_shares = totals.share
        FROM totals
        WHERE totals.work_id = works.id;
        ";
    _ = conn.execute(sql, params!())?;
    Ok(())
}

/// save writer -> publisher and publisher -> publisher instances
pub fn enrich_publisher_relations(conn: &rusqlite::Connection) -> Result<(), rusqlite::Error> {
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
    _ = conn.execute(sql, params!())?;
    Ok(())
}

/// save writer -> writer instances
pub fn enrich_writer_relations(conn: &rusqlite::Transaction) -> Result<(), rusqlite::Error> {
    let sql = "
          WITH root_writer_shares AS (
            SELECT DISTINCT
                s.work_id,
                s.party_id
            FROM shares s NOT INDEXED
            WHERE role IN (
                'ComposerLyricist',
                'Composer',
                'Lyricist',
                'Arranger',
                'SubLyricist',
                'Adapter',
                'Translator',
                'SubArranger'
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
    _ = conn.execute(sql, params!())?;
    Ok(())
}
