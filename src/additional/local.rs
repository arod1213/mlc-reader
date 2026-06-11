type ResType = Result<(), sqlite::Error>;

pub fn wrap_tx(conn: &sqlite::Connection, f: fn(&sqlite::Connection) -> ResType) -> ResType {
    conn.execute("BEGIN TRANSACTION")?;
    match f(conn) {
        Ok(_) => {
            conn.execute("COMMIT")?;
            Ok(())
        }
        Err(e) => {
            conn.execute("ROLLBACK")?;
            Err(e)
        }
    }
}

pub fn enrich_publisher_relations(conn: &sqlite::Connection) -> Result<(), sqlite::Error> {
    let sql = "
        INSERT INTO publisher_relations (parent_id, child_id)
        SELECT DISTINCT
            p.id  AS parent_id,
            rp.id AS child_id
        FROM parties p
        JOIN shares s  ON s.party_id = p.id
        JOIN shares rs ON rs.preceding_id = s.id
        JOIN parties rp ON rp.id = rs.party_id
        ON CONFLICT(parent_id, child_id) DO UPDATE
        SET occurrences = occurrences + 1;
        ";
    conn.execute(sql)?;
    Ok(())
}

pub fn enrich_writer_relations(conn: &sqlite::Connection) -> Result<(), sqlite::Error> {
    let sql = "
          INSERT INTO writer_relations (writer_a, writer_b)
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
          )
          SELECT
              a.party_id AS writer_a,
              b.party_id AS writer_b
          FROM root_writer_shares a
          JOIN root_writer_shares b
            ON a.work_id = b.work_id
           AND a.party_id < b.party_id
           ON CONFLICT(writer_a, writer_b) DO UPDATE
           SET occurrences = occurrences + 1;
        ";
    conn.execute(sql)?;
    Ok(())
}
