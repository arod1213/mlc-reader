use rusqlite::{Connection, params};

pub fn disable_fk(conn: &Connection) -> Result<(), rusqlite::Error> {
    _ = conn.execute("PRAGMA foreign_keys = OFF", params!())?;
    Ok(())
}

pub fn setup_write_mode(conn: &Connection) -> Result<(), rusqlite::Error> {
    let mut stmt = conn.prepare("PRAGMA journal_mode = WAL")?;
    let mut rows = stmt.query([])?;
    while rows.next()?.is_some() {}

    conn.execute("PRAGMA synchronous = NORMAL", params!())?;
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

fn run_pragma(conn: &Connection, sql: &str) -> Result<(), rusqlite::Error> {
    let mut stmt = conn.prepare(sql)?;
    let mut rows = stmt.query([])?;
    while rows.next()?.is_some() {}
    Ok(())
}
