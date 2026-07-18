// pub fn search_writer_relations(&self, party_id: i64) {}
// pub fn search_publisher_relations(&self, party_id: i64) {}
// pub fn audit_party_catalog(&self, party_id: i64) {}
// }
//

use std::str::FromStr;

use crate::types::{Party, Role};
use libsql::{Connection, params};
use musicmeta::{ipi::IpiNameNum, tis::society::TisSocietyCode};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum PartyInput {
    Name(String),
    Ipi(IpiNameNum),
}

impl From<String> for PartyInput {
    fn from(value: String) -> Self {
        match value
            .parse::<u64>()
            .ok()
            .and_then(|x| IpiNameNum::new(x).ok())
        {
            Some(ipi) => PartyInput::Ipi(ipi),
            None => PartyInput::Name(value),
        }
    }
}

/// searches writers / publishers by name or ipi
/// skips writers missing IPI
pub async fn search_parties(
    conn: &Connection,
    param: PartyInput,
    role: Option<Role>,
    limit: usize,
) -> Result<Vec<Party>, libsql::Error> {
    let mut rows = match param {
        PartyInput::Name(name) => {
            if name.is_empty() {
                return Err(libsql::Error::Misuse(
                    "search name must not be empty".to_string(),
                ));
            }
            let clean_name: String = name
                .chars()
                .map(|x| {
                    if x.is_ascii_digit() || x.is_alphabetic() {
                        x
                    } else {
                        ' '
                    }
                })
                .collect();

            match &role {
                Some(x) => {
                    let sql = "
                        SELECT p.id, p.first_name, p.last_name, p.ipi, p.pro, p.role
                        FROM parties p
                        JOIN parties_fts ON parties_fts.rowid = p.id
                        WHERE p.ipi IS NOT NULL
                        AND parties_fts MATCH ?
                        AND (p.role = ? OR p.role IS NULL OR p.role = 'both')
                        LIMIT ?";
                    conn.query(sql, params!(clean_name, x.to_string(), limit as i64))
                        .await?
                }
                None => {
                    let sql = "
                        SELECT p.id, p.first_name, p.last_name, p.ipi, p.pro, p.role
                        FROM parties p
                        JOIN parties_fts ON parties_fts.rowid = p.id
                        WHERE p.ipi IS NOT NULL
                        AND parties_fts MATCH ?
                        LIMIT ?";
                    conn.query(sql, params!(clean_name, limit as i64)).await?
                }
            }
        }
        PartyInput::Ipi(ipi) => match &role {
            Some(x) => {
                let sql = "
                SELECT p.id, p.first_name, p.last_name, p.ipi, p.pro, p.role
                FROM parties p
                WHERE p.ipi = ?
                AND (p.role = ? OR p.role IS NULL OR p.role = 'both')
                LIMIT ?";
                conn.query(sql, params!(ipi.0 as i64, x.to_string(), limit as i64))
                    .await?
            }
            None => {
                let sql = "
                SELECT p.id, p.first_name, p.last_name, p.ipi, p.pro, p.role
                FROM parties p
                WHERE p.ipi = ?
                LIMIT ?";
                conn.query(sql, params!(ipi.0 as i64, limit as i64)).await?
            }
        },
    };

    let mut v = vec![];
    while let Some(row) = rows.next().await? {
        let w = Party {
            id: row.get::<u64>(0)?,
            first_name: row.get(1).ok(),
            last_name: row.get(2)?,
            ipi_name_num: row
                .get::<i64>(3)
                .ok()
                .and_then(|x| IpiNameNum::new(x as u64).ok()),
            pro: row
                .get::<u64>(4)
                .ok()
                .and_then(|v| TisSocietyCode::try_from(v as u16).ok()),
            role: row
                .get::<String>(5)
                .and_then(|s| Role::from_str(&s).map_err(|_| libsql::Error::NullValue))
                .ok(),
        };
        v.push(w);
    }
    Ok(v)
}

pub async fn top_unsigned_writers(conn: &Connection) -> Result<Vec<Party>, libsql::Error> {
    let sql = "
  WITH latest_share AS (
      SELECT s.id, p.id AS party_id
      FROM shares s
      JOIN parties p ON p.id = s.party_id
      JOIN work_resources wr ON wr.work_id = s.work_id
      JOIN resources rs ON rs.id = wr.resource_id
      JOIN releases r ON r.id = rs.release_id
      GROUP BY p.id, s.id
      ORDER BY s.id DESC, r.release_date DESC
  )
  SELECT
      p.id,
      p.first_name,
      p.last_name,
      p.pro,
      p.ipi
  FROM parties p
  WHERE
      (role IS NULL OR role IN ('writer', 'both'))
      AND p.work_count > 0
      AND (
          ?1 IS NULL OR (
              p.average_share > ?1
          )
      )
      AND NOT EXISTS (
          SELECT 1
          FROM shares s
          WHERE s.preceding_id = (
              SELECT ls.id
              FROM latest_share ls
              WHERE ls.party_id = p.id
          )
      )
  ORDER BY p.average_share DESC";
    let mut rows = conn.query(sql, params!(20.0)).await?;
    let mut v = vec![];
    while let Some(row) = rows.next().await? {
        v.push(Party {
            id: row.get(0)?,
            first_name: row.get(1)?,
            last_name: row.get(2)?,
            ipi_name_num: row
                .get::<i64>(5)
                .ok()
                .and_then(|x| IpiNameNum::new(x as u64).ok()),
            pro: None,
            role: None,
        })
    }
    Ok(v)
}
