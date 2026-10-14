use rusqlite::params;

pub fn create_share_pk(conn: &rusqlite::Connection) -> Result<(), rusqlite::Error> {
    let sql = "CREATE UNIQUE INDEX IF NOT EXISTS shares_id_idx ON shares(id);";
    _ = conn.execute(sql, params!())?;
    Ok(())
}

pub fn create_work_pk(conn: &rusqlite::Connection) -> Result<(), rusqlite::Error> {
    let sql = "CREATE UNIQUE INDEX IF NOT EXISTS works_id_idx ON works(id);";
    _ = conn.execute(sql, params!())?;
    Ok(())
}

pub fn create_party_pk(conn: &rusqlite::Connection) -> Result<(), rusqlite::Error> {
    let sql = "CREATE UNIQUE INDEX IF NOT EXISTS parties_id_idx ON parties(id);";
    _ = conn.execute(sql, params!())?;
    Ok(())
}

pub fn create_resource_pk(conn: &rusqlite::Connection) -> Result<(), rusqlite::Error> {
    let sql = "CREATE UNIQUE INDEX IF NOT EXISTS resources_id_idx ON resources(id);";
    _ = conn.execute(sql, params!())?;
    Ok(())
}
