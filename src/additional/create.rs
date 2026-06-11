use libsql::params;

pub async fn assign_roles(conn: &libsql::Transaction) -> Result<(), libsql::Error> {
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
    _ = conn.execute(sql, params!()).await?;
    Ok(())
}
