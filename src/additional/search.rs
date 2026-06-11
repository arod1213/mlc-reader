use libsql::{Connection, params};

pub struct Writer {
    pub name: String,
    pub ipi: i64,
    pub pro: String,
}

pub async fn find_unsigned_writers(conn: &Connection) -> Result<Vec<Writer>, libsql::Error> {
    let sql = "SELECT * FROM parties";
    let mut rows = conn.query(sql, params!()).await?;
    let mut v = vec![];
    loop {
        let Some(row) = rows.next().await? else { break };
        v.push(Writer {
            name: row.get(0)?,
            ipi: row.get(1)?,
            pro: row.get(2)?,
        })
    }
    Ok(v)
}
