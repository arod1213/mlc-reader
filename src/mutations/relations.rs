use std::str::FromStr;

use crate::types::{Party, Role};
use libsql::{Connection, params};
use musicmeta::{ipi::IpiNameNum, tis::society::TisSocietyCode};

pub enum RelatedPartyInput {
    Id(String),
    Ipi(IpiNameNum),
}
impl From<String> for RelatedPartyInput {
    fn from(value: String) -> Self {
        match value
            .parse::<u64>()
            .ok()
            .and_then(|x| IpiNameNum::new(x).ok())
        {
            Some(ipi) => RelatedPartyInput::Ipi(ipi),
            None => RelatedPartyInput::Id(value),
        }
    }
}

/// fetches parties that the ID has as a preceeding share ID
pub async fn get_party_publishers(
    conn: &Connection,
    key: RelatedPartyInput,
) -> Result<Vec<Party>, libsql::Error> {
    let mut rows = match key {
        RelatedPartyInput::Id(id) => {
            let sql = "
                SELECT
                      p.id         AS id,
                      p.first_name AS first_name,
                      p.full_name  AS full_name,
                      p.last_name  AS last_name,
                      p.ipi        AS ipi,
                      p.pro        AS pro,
                      p.role       AS role
                  FROM publisher_relations w
                  JOIN parties p on p.id = w.child_id
                  WHERE w.parent_id = ?
                  ORDER BY w.occurrences DESC
                  LIMIT 10";
            conn.query(sql, params![id]).await?
        }
        RelatedPartyInput::Ipi(ipi_name_num) => {
            let sql = "
                SELECT
                      p.id         AS id,
                      p.first_name AS first_name,
                      p.full_name  AS full_name,
                      p.last_name  AS last_name,
                      p.ipi        AS ipi,
                      p.pro        AS pro,
                      p.role       AS role
                  FROM publisher_relations w
                  JOIN parties p on p.id = w.child_id
                  JOIN parties pw on pw.id = w.parent_id
                  WHERE pw.ipi = ?
                  ORDER BY w.occurrences DESC
                  LIMIT 10";
            conn.query(sql, params![ipi_name_num.0 as i64]).await?
        }
    };
    let mut v = vec![];
    while let Some(row) = rows.next().await? {
        let (first, last) = {
            let a: Option<String> = row.get(1)?;
            let b: String = row.get(3)?;
            if (a.is_none() || a == Some("".into())) && b.is_empty() {
                (None, row.get(2)?)
            } else {
                (a, b)
            }
        };
        let w = Party {
            id: row.get::<u64>(0).ok(),
            first_name: first,
            last_name: last,
            ipi_name_num: row
                .get::<i64>(4)
                .ok()
                .and_then(|x| IpiNameNum::new(x as u64).ok()),
            pro: row
                .get::<i64>(5)
                .ok()
                .and_then(|v| TisSocietyCode::try_from(v as u16).ok()),
            role: row
                .get::<String>(6)
                .and_then(|s| Role::from_str(&s).map_err(|_| libsql::Error::NullValue))
                .ok(),
        };
        v.push(w);
    }
    Ok(v)
}

/// fetches parties that the ID represents as preceeding share ID
pub async fn get_party_writers(
    conn: &Connection,
    key: RelatedPartyInput,
) -> Result<Vec<Party>, libsql::Error> {
    let mut rows = match key {
        RelatedPartyInput::Id(id) => {
            let sql = "
                SELECT
                      p.id         AS id,
                      p.first_name AS first_name,
                      p.full_name  AS full_name,
                      p.last_name  AS last_name,
                      p.ipi        AS ipi,
                      p.pro        AS pro,
                      p.role       AS role
                  FROM publisher_relations w
                  JOIN parties p on p.id = w.parent_id
                  WHERE w.child_id = ?
                  ORDER BY w.occurrences DESC
                  LIMIT 10";

            conn.query(sql, params![id]).await?
        }
        RelatedPartyInput::Ipi(ipi_name_num) => {
            let sql = "
                SELECT
                      p.id         AS id,
                      p.first_name AS first_name,
                      p.full_name  AS full_name,
                      p.last_name  AS last_name,
                      p.ipi        AS ipi,
                      p.pro        AS pro,
                      p.role       AS role
                  FROM publisher_relations w
                  JOIN parties p on p.id = w.parent_id
                  JOIN parties wp on wp.id = w.child_id
                  WHERE wp.ipi = ?
                  ORDER BY w.occurrences DESC
                  LIMIT 10";

            conn.query(sql, params![ipi_name_num.0 as i64]).await?
        }
    };
    let mut v = vec![];
    while let Some(row) = rows.next().await? {
        let (first, last) = {
            let a: Option<String> = row.get(1)?;
            let b: String = row.get(3)?;
            if (a.is_none() || a == Some("".into())) && b.is_empty() {
                (None, row.get(2)?)
            } else {
                (a, b)
            }
        };
        let w = Party {
            id: row.get::<u64>(0).ok(),
            first_name: first,
            last_name: last,
            ipi_name_num: row
                .get::<i64>(4)
                .ok()
                .and_then(|x| IpiNameNum::new(x as u64).ok()),
            pro: row
                .get::<u64>(5)
                .ok()
                .and_then(|v| TisSocietyCode::try_from(v as u16).ok()),
            role: row
                .get::<String>(6)
                .and_then(|s| Role::from_str(&s).map_err(|_| libsql::Error::NullValue))
                .ok(),
        };
        v.push(w);
    }
    Ok(v)
}

/// fetches top writing partners for a given writer
pub async fn get_writer_collaborators(
    conn: &Connection,
    party_id: i64,
    offset: usize,
) -> Result<Vec<Party>, libsql::Error> {
    let sql = "
          SELECT
            p.id,
            p.first_name,
            p.full_name,
            p.last_name,
            p.ipi,
            p.pro,
            p.role,
            u.occurrences
          FROM (
            SELECT wr.writer_b AS other_writer_id, wr.occurrences
            FROM writer_relations wr
            WHERE wr.writer_a = $1

            UNION ALL

            SELECT wr.writer_a AS other_writer_id, wr.occurrences
            FROM writer_relations wr
            WHERE wr.writer_b = $1
          ) u
          JOIN parties p
            ON p.id = u.other_writer_id
           AND (p.role = 'writer' OR p.role IS NULL)
          ORDER BY u.occurrences DESC
          LIMIT 15 OFFSET $2;
        ";

    let mut rows = conn.query(sql, params!(party_id, offset as i64)).await?;

    let mut v = vec![];
    while let Some(row) = rows.next().await? {
        let (first, last) = {
            let a: Option<String> = row.get(1)?;
            let b: String = row.get(3)?;
            if (a.is_none() || a == Some("".into())) && b.is_empty() {
                (None, row.get(2)?)
            } else {
                (a, b)
            }
        };
        let w = Party {
            id: row.get::<u64>(0).ok(),
            first_name: first,
            last_name: last,
            ipi_name_num: row
                .get::<i64>(4)
                .ok()
                .and_then(|x| IpiNameNum::new(x as u64).ok()),
            pro: row
                .get::<u64>(5)
                .ok()
                .and_then(|v| TisSocietyCode::try_from(v as u16).ok()),
            role: row
                .get::<String>(6)
                .and_then(|s| Role::from_str(&s).map_err(|_| libsql::Error::NullValue))
                .ok(),
        };
        v.push(w);
    }
    Ok(v)
}
