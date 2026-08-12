use libsql::{Connection, params};

async fn run_pragma(conn: &Connection, sql: &str) -> Result<(), libsql::Error> {
    let mut rows = conn.query(sql, params!()).await?;
    while rows.next().await?.is_some() {}
    Ok(())
}

pub async fn setup_bulk_write_mode(conn: &Connection) -> Result<(), libsql::Error> {
    run_pragma(conn, "PRAGMA foreign_keys = OFF").await?;
    run_pragma(conn, "PRAGMA journal_mode = WAL").await?;
    run_pragma(conn, "PRAGMA synchronous = OFF").await?;
    run_pragma(conn, "PRAGMA temp_store = MEMORY").await?;
    run_pragma(conn, "PRAGMA cache_size = -2000000").await?;
    run_pragma(conn, "PRAGMA mmap_size = 30000000000").await?;
    run_pragma(conn, "PRAGMA busy_timeout = 60000").await?;
    Ok(())
}

pub async fn setup_trim_write_mode(conn: &Connection) -> Result<(), libsql::Error> {
    run_pragma(conn, "PRAGMA foreign_keys = OFF").await?;
    run_pragma(conn, "PRAGMA journal_mode = WAL").await?;
    run_pragma(conn, "PRAGMA synchronous = OFF").await?;
    run_pragma(conn, "PRAGMA temp_store = FILE").await?;
    run_pragma(conn, "PRAGMA cache_size = -200000").await?;
    run_pragma(conn, "PRAGMA mmap_size = 1073741824").await?;
    run_pragma(conn, "PRAGMA busy_timeout = 60000").await?;
    Ok(())
}

pub async fn trim_works(conn: &Connection) -> Result<(), libsql::Error> {
    let sql = "
        DELETE FROM works
        WHERE 
          NOT EXISTS (
            SELECT 1 FROM shares s WHERE s.work_id = works.id
          )
        AND
          NOT EXISTS (
            SELECT 1 FROM resources r
            JOIN work_resources wr on wr.resource_id = r.id
            WHERE wr.work_id = works.id
          )
    ";
    conn.execute(sql, params!()).await?;
    Ok(())
}

pub async fn trim_shares(conn: &Connection) -> Result<(), libsql::Error> {
    let sql = "
        DELETE FROM shares
        WHERE shares.share = 0.0
        AND NOT EXISTS (
          SELECT 1 FROM shares s
          WHERE s.preceding_id = shares.id
        )
    ";
    conn.execute(sql, params!()).await?;
    Ok(())
}

pub async fn trim_releases(conn: &Connection) -> Result<(), libsql::Error> {
    let sql = "
        DELETE FROM releases
        WHERE NOT EXISTS (
            SELECT 1 FROM resources r
            INNER JOIN work_resources wr on wr.resource_id = r.id
            WHERE r.release_id = releases.id
        )
    ";
    conn.execute(sql, params!()).await?;
    Ok(())
}

pub async fn trim_parties(conn: &Connection) -> Result<(), libsql::Error> {
    let sql = "
        DELETE FROM parties
        WHERE NOT EXISTS (
            SELECT 1 FROM shares s
            WHERE s.party_id = parties.id
        )
        AND parties.ipi IS NULL
    ";
    conn.execute(sql, params!()).await?;
    Ok(())
}

// async fn delete_old_works(conn: &Connection) -> Result<(), libsql::Error> {
//     let sql = "
//         WITH old_releases AS (
//           SELECT id
//           FROM releases
//           WHERE release_date < unixepoch('2015-01-01')
//         ),
//         old_works AS (
//           SELECT wk.id
//           FROM works wk
//           WHERE EXISTS (
//             SELECT 1
//             FROM old_releases r
//             JOIN resources rs ON rs.release_id = r.id
//             JOIN work_resources wr ON wr.resource_id = rs.id
//             WHERE wr.work_id = wk.id
//           )
//         )
//         DELETE FROM works
//         WHERE id IN (SELECT id FROM old_works);
//     ";
//     conn.execute(sql, params!()).await?;
//     Ok(())
// }
