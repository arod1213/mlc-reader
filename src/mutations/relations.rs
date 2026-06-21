use sqlite::{Connection, State};

/// fetch writers collaborators
pub fn get_writer_collaborators(conn: &Connection, id: i64) -> Result<(), sqlite::Error> {
    let sql = "
      SELECT DISTINCT p.id as id, p.full_name as full_name
      FROM writer_relations wr
      JOIN parties p ON p.id = wr.writer_b AND (p.role = 'writer' OR p.role is NULL)
      WHERE wr.writer_a = @id

      UNION
      SELECT DISTINCT p.id as id, p.full_name as full_name
      FROM writer_relations wr
      JOIN parties p ON p.id = wr.writer_a AND (p.role = 'writer' OR p.role is NULL)
      WHERE wr.writer_b = @id;
        ";

    let mut stmt = conn.prepare(sql)?;
    stmt.bind::<&[(_, sqlite::Value)]>(&[("@id", id.into())])?;

    while let State::Row = stmt.next()? {
        let id: i64 = stmt.read("id")?;
        let full_name: String = stmt.read("full_name")?;
        println!("{}: {}", id, full_name);
    }
    Ok(())
}
