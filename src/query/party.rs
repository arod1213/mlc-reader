use std::str::FromStr;

use musicmeta::tis::society::TisSocietyCode;
use rusqlite::{ToSql, params, params_from_iter};
use serde::Deserialize;

use crate::query::types::{Party, PartyInput, PartyMeta, Role};

pub fn get_party(db: &rusqlite::Connection, id: u64) -> Result<Party, rusqlite::Error> {
    let q = "
        SELECT 
            id, 
            first_name, 
            last_name, 
            ipi, 
            pro, 
            role
        FROM parties
        WHERE id = ?
        ";

    let party = db.query_one(q, params![id as i64], |row| {
        Ok(Party {
            id,
            first_name: row.get(1)?,
            last_name: row.get(2)?,
            ipi_name_num: row.get::<_, Option<i64>>(3)?.map(|x| x as u64),
            pro: row
                .get::<_, Option<i64>>(4)?
                .and_then(|v| TisSocietyCode::try_from(v as u16).ok()),
            role: row
                .get::<_, Option<String>>(5)?
                .and_then(|x| Role::from_str(&x).ok()),
        })
    })?;
    Ok(party)
}

pub fn search_parties(
    db: &rusqlite::Connection,
    key: PartyInput,
) -> Result<Vec<Party>, rusqlite::Error> {
    let limit = 50;
    match key {
        PartyInput::Name(name) => {
            if name.is_empty() {
                return Err(rusqlite::Error::InvalidQuery);
            }

            let q = "
            SELECT 
                p.id, 
                p.first_name, 
                p.full_name,
                p.last_name, 
                p.ipi, 
                p.pro, 
                p.role
            FROM parties p
            JOIN parties_fts ON parties_fts.rowid = p.id
            WHERE parties_fts MATCH ?1 || '*'
            AND p.ipi IS NOT NULL
            LIMIT ?2";

            let mut stmt = db.prepare(q)?;
            stmt.query_map(params![name, limit], |row| {
                let (first, last) = {
                    let a: Option<String> = row.get(1)?;
                    let b: String = row.get(3)?;
                    if (a.is_none() || a == Some("".into())) && b.is_empty() {
                        (None, row.get(2)?)
                    } else {
                        (a, b)
                    }
                };

                Ok(Party {
                    id: row.get::<_, i64>(0)? as u64,
                    first_name: first,
                    last_name: last,
                    ipi_name_num: row.get::<_, Option<i64>>(4)?.map(|x| x as u64),
                    pro: row
                        .get::<_, Option<i64>>(5)?
                        .and_then(|v| TisSocietyCode::try_from(v as u16).ok()),
                    role: row
                        .get::<_, Option<String>>(6)?
                        .and_then(|s| Role::from_str(&s).ok()),
                })
            })?
            .collect()
        }
        PartyInput::Ipi(ipi) => {
            let q = "
            SELECT 
                id, 
                first_name, 
                full_name,
                last_name, 
                ipi, 
                pro, 
                role
            FROM parties
            WHERE ipi = ?1
            LIMIT ?2";

            log::info!("searching {ipi} by IPI");
            let mut stmt = db.prepare(q)?;
            stmt.query_map(params![ipi.0 as i64, limit], |row| {
                let (first, last) = {
                    let a: Option<String> = row.get(1)?;
                    let b: String = row.get(3)?;
                    if (a.is_none() || a == Some("".into())) && b.is_empty() {
                        (None, row.get(2)?)
                    } else {
                        (a, b)
                    }
                };

                Ok(Party {
                    id: row.get::<_, i64>(0)? as u64,
                    first_name: first,
                    last_name: last,
                    ipi_name_num: row.get::<_, Option<i64>>(4)?.map(|x| x as u64),
                    pro: row
                        .get::<_, Option<i64>>(5)?
                        .and_then(|v| TisSocietyCode::try_from(v as u16).ok()),
                    role: row
                        .get::<_, Option<String>>(6)?
                        .and_then(|s| Role::from_str(&s).ok()),
                })
            })?
            .collect()
        }
    }
}

pub fn get_party_collaborators(
    db: &rusqlite::Connection,
    party_id: u64,
    is_publishers: bool,
) -> Result<Vec<Party>, rusqlite::Error> {
    let limit = 50;
    let q = if is_publishers {
        "SELECT
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
          LIMIT ?"
    } else {
        "SELECT
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
          LIMIT ?"
    };
    let mut stmt = db.prepare(q)?;
    let rows = stmt.query_map(params![party_id as i64, limit], |row| {
        let (first, last) = {
            let a: Option<String> = row.get(1)?;
            let b: String = row.get(3)?;
            if (a.is_none() || a == Some("".into())) && b.is_empty() {
                (None, row.get(2)?)
            } else {
                (a, b)
            }
        };
        Ok(Party {
            id: row.get::<_, i64>(0)? as u64,
            first_name: first,
            last_name: last,
            ipi_name_num: row.get::<_, Option<i64>>(4)?.map(|x| x as u64),
            pro: row
                .get::<_, Option<i64>>(5)?
                .and_then(|v| TisSocietyCode::try_from(v as u16).ok()),
            role: row
                .get::<_, Option<String>>(6)?
                .and_then(|s| Role::from_str(&s).ok()),
        })
    })?;
    rows.collect()
}

pub fn parties_by_market_share<T: AsRef<str> + ToSql>(
    db: &rusqlite::Connection,
    isrcs: &[T],
) -> Result<Vec<PartyMeta>, rusqlite::Error> {
    let placeholders = vec!["?"; isrcs.len()].join(", ");
    let q = format!(
        "
        WITH matched_resources AS (
            SELECT id, isrc
            FROM resources
            WHERE isrc IN ({placeholders})
        ), matched_works AS (
            SELECT
              r.isrc,
              wk.id as id
            FROM matched_resources r
            JOIN work_resources wr ON wr.resource_id = r.id
            JOIN works wk on wk.id = wr.work_id
        )
        SELECT 
            s.party_id,
            COUNT(*) as num_works,
            SUM(s.share) as total_shares
        FROM shares s
        WHERE s.work_id IN (SELECT mw.id FROM matched_works mw)
        GROUP BY 1
        "
    );

    let mut stmt = db.prepare(&q)?;
    let rows = stmt.query_map(params_from_iter(isrcs.iter()), |row| {
        let num_works: usize = row.get::<_, i64>(1)? as usize;
        let total_shares: f32 = row.get(2)?;
        let avg_share = if num_works == 0 {
            0.0
        } else {
            total_shares / (num_works as f32)
        };
        Ok(PartyMeta {
            party_id: row.get::<_, i64>(0)? as u64,
            isrcs: vec![],
            num_works,
            total_shares,
            avg_share,
        })
    })?;

    rows.collect()
}
