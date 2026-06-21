use libsql::{Connection, Error, Transaction, params};
use musicmeta::tis::society::TisSocietyCode;

/// set PRO for publisher and all associated writers
pub async fn update_publisher_writers(
    conn: &Connection,
    id: i64,
    pro_code: TisSocietyCode,
) -> Result<(), Error> {
    let tx = conn.transaction().await.unwrap();
    let is_err = match update_publisher_writers_inner(&tx, id, pro_code).await {
        Ok(_) => false,
        Err(e) => {
            eprintln!("{:?}", e);
            true
        }
    };
    match is_err {
        true => tx.rollback().await,
        false => tx.commit().await,
    }
}

async fn update_publisher_writers_inner(
    conn: &Transaction,
    id: i64,
    pro_code: TisSocietyCode,
) -> Result<(), libsql::Error> {
    update_writers(conn, id, &pro_code).await?;

    let sql = "
        UPDATE parties SET pro = ?
        WHERE id = ?";
    let stmt = conn.prepare(sql).await?;
    stmt.execute(params![pro_code as i64, id]).await?;
    Ok(())
}

async fn update_writers(
    conn: &Connection,
    id: i64,
    pro_code: &TisSocietyCode,
) -> Result<(), libsql::Error> {
    let sql = "
        UPDATE parties SET pro = ?
        WHERE id IN (
            SELECT parent_id
            FROM publisher_relations
            WHERE parent_id = ?
        )";
    let stmt = conn.prepare(sql).await?;
    stmt.execute(params![(*pro_code).clone() as i64, id])
        .await?;
    Ok(())
}
