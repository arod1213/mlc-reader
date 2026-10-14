use musicmeta::tis::society::TisSocietyCode;
use rusqlite::{Connection, Error, Transaction, params};

/// set PRO for publisher and all associated writers
pub fn update_publisher_writers(
    conn: &mut Connection,
    id: i64,
    pro_code: TisSocietyCode,
) -> Result<(), Error> {
    let tx = conn.transaction().unwrap();
    let is_err = match update_publisher_writers_inner(&tx, id, pro_code) {
        Ok(_) => false,
        Err(e) => {
            log::error!("{:?}", e);
            true
        }
    };
    match is_err {
        true => tx.rollback(),
        false => tx.commit(),
    }
}

fn update_publisher_writers_inner(
    conn: &Transaction,
    id: i64,
    pro_code: TisSocietyCode,
) -> Result<(), rusqlite::Error> {
    update_writers(conn, id, &pro_code)?;

    let sql = "
        UPDATE parties SET pro = ?
        WHERE id = ?";
    let mut stmt = conn.prepare(sql)?;
    stmt.execute(params![pro_code as i64, id])?;
    Ok(())
}

fn update_writers(
    conn: &Connection,
    id: i64,
    pro_code: &TisSocietyCode,
) -> Result<(), rusqlite::Error> {
    let sql = "
        UPDATE parties SET pro = ?
        WHERE id IN (
            SELECT parent_id
            FROM publisher_relations
            WHERE parent_id = ?
        )";
    let mut stmt = conn.prepare(sql)?;
    stmt.execute(params![(*pro_code).clone() as i64, id])?;
    Ok(())
}
