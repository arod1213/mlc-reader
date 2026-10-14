use rusqlite::{Connection, params};

fn run_pragma(conn: &Connection, sql: &str) -> Result<(), rusqlite::Error> {
    let mut stmt = conn.prepare(sql)?;
    let mut rows = stmt.query([])?;
    while rows.next()?.is_some() {}
    Ok(())
}

pub fn setup_bulk_write_mode(conn: &Connection) -> Result<(), rusqlite::Error> {
    run_pragma(conn, "PRAGMA foreign_keys = OFF")?;
    run_pragma(conn, "PRAGMA journal_mode = WAL")?;
    run_pragma(conn, "PRAGMA synchronous = OFF")?;
    run_pragma(conn, "PRAGMA temp_store = MEMORY")?;
    run_pragma(conn, "PRAGMA cache_size = -2000000")?;
    run_pragma(conn, "PRAGMA mmap_size = 30000000000")?;
    run_pragma(conn, "PRAGMA busy_timeout = 60000")?;
    Ok(())
}

pub fn setup_trim_write_mode(conn: &Connection) -> Result<(), rusqlite::Error> {
    run_pragma(conn, "PRAGMA foreign_keys = OFF")?;
    run_pragma(conn, "PRAGMA journal_mode = WAL")?;
    run_pragma(conn, "PRAGMA synchronous = OFF")?;
    run_pragma(conn, "PRAGMA temp_store = FILE")?;
    run_pragma(conn, "PRAGMA cache_size = -200000")?;
    run_pragma(conn, "PRAGMA mmap_size = 1073741824")?;
    run_pragma(conn, "PRAGMA busy_timeout = 60000")?;
    Ok(())
}

pub fn trim_works(conn: &Connection) -> Result<usize, rusqlite::Error> {
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
    conn.execute(sql, params!())
}

pub fn trim_shares(conn: &Connection) -> Result<usize, rusqlite::Error> {
    let sql = "
        DELETE FROM shares
        WHERE shares.share = 0.0
        AND NOT EXISTS (
          SELECT 1 FROM shares s
          WHERE s.preceding_id = shares.id
        )
    ";
    conn.execute(sql, params!())
}

pub fn trim_releases(conn: &Connection) -> Result<usize, rusqlite::Error> {
    let sql = "
        DELETE FROM releases
        WHERE NOT EXISTS (
            SELECT 1 FROM resources r
            INNER JOIN work_resources wr on wr.resource_id = r.id
            WHERE r.release_id = releases.id
        )
    ";
    conn.execute(sql, params!())
}

pub fn trim_parties(conn: &Connection) -> Result<usize, rusqlite::Error> {
    let sql = "
        DELETE FROM parties
        WHERE NOT EXISTS (
            SELECT 1 FROM shares s
            WHERE s.party_id = parties.id
        )
        AND parties.ipi IS NULL
    ";
    conn.execute(sql, params!())
}
