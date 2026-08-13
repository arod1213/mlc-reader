use libsql::params;

pub async fn create_share_pk(conn: &libsql::Connection) -> Result<(), libsql::Error> {
    let sql = "CREATE UNIQUE INDEX IF NOT EXISTS shares_id_idx ON shares(id);";
    _ = conn.execute(sql, params!()).await?;
    Ok(())
}

pub async fn create_work_pk(conn: &libsql::Connection) -> Result<(), libsql::Error> {
    let sql = "CREATE UNIQUE INDEX IF NOT EXISTS works_id_idx ON works(id);";
    _ = conn.execute(sql, params!()).await?;
    Ok(())
}

pub async fn create_party_pk(conn: &libsql::Connection) -> Result<(), libsql::Error> {
    let sql = "CREATE UNIQUE INDEX IF NOT EXISTS parties_id_idx ON parties(id);";
    _ = conn.execute(sql, params!()).await?;
    Ok(())
}

pub async fn create_resource_pk(conn: &libsql::Connection) -> Result<(), libsql::Error> {
    let sql = "CREATE UNIQUE INDEX IF NOT EXISTS resources_id_idx ON resources(id);";
    _ = conn.execute(sql, params!()).await?;
    Ok(())
}
