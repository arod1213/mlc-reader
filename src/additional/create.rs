use sqlite::Connection;

pub fn assign_roles(conn: &Connection) -> Result<(), sqlite::Error> {
    let sql = "
      UPDATE parties
      SET role = CASE
        WHEN (
          SELECT COUNT(*)
          FROM publisher_relations
          WHERE child_id = parties.id
        ) > (
          SELECT COUNT(*)
          FROM publisher_relations
          WHERE parent_id = parties.id
        ) THEN 'writer'
        WHEN (
          SELECT COUNT(*)
          FROM publisher_relations
          WHERE parent_id = parties.id
        ) > (
          SELECT COUNT(*)
          FROM publisher_relations
          WHERE child_id = parties.id
        ) THEN 'publisher'
        WHEN (
          SELECT COUNT(*)
          FROM publisher_relations
          WHERE parent_id = parties.id
        ) > 0 THEN 'both'
        ELSE NULL
      END;
        ";
    conn.execute(sql)?;
    Ok(())
}
