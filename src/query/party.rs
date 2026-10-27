use std::{collections::HashSet, str::FromStr};

use musicmeta::tis::society::TisSocietyCode;
use rusqlite::{Connection, ToSql, params, params_from_iter};

use crate::query::types::{Party, PartyInput, PartyMeta, Role};

// pub fn party_bfs(db: &Connection, a_id: u64, b_id: u64) -> Option<Vec<u64>> {
//     let Some(a_rel) = party_collabs(db, a_id) else {
//         return None;
//     }
//     let Some(b_rel) = party_collabs(db, b_id) else {
//         return None;
//     }
//     None
// }

// fn party_collabs(db: &Connection, id: u64) -> Option<HashSet<u64>> {
//     let q = "
//         SELECT writer_b
//         FROM writer_relations
//         WHERE writer_a = ?1
//
//         UNION
//
//         SELECT writer_a
//         FROM writer_relations
//         WHERE writer_b = ?1
//     ";
//     let mut stmt = db.prepare(q).ok()?;
//     let rows = stmt
//         .query_map(params!(id as i64), |row| {
//             let id = row.get::<_, i64>(0)?;
//             Ok(id as u64)
//         })
//         .ok()?;
//     rows.collect::<Result<HashSet<u64>, _>>().ok()
// }

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
) -> Result<Vec<Party>, rusqlite::Error> {
    let limit = 300;
    let q = "
        SELECT
              p.id         AS id,
              p.first_name AS first_name,
              p.full_name  AS full_name,
              p.last_name  AS last_name,
              p.ipi        AS ipi,
              p.pro        AS pro,
              p.role       AS role
          FROM writer_relations w
          JOIN parties p on p.id = w.writer_a
          WHERE w.writer_b = ?1

        UNION

        SELECT
              p.id         AS id,
              p.first_name AS first_name,
              p.full_name  AS full_name,
              p.last_name  AS last_name,
              p.ipi        AS ipi,
              p.pro        AS pro,
              p.role       AS role
          FROM writer_relations w
          JOIN parties p on p.id = w.writer_b
          WHERE w.writer_a = ?1
          ORDER BY full_name ASC
          LIMIT ?2
      ";
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

pub fn get_party_chain_of_title(
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
        WITH matched_works AS (
            SELECT DISTINCT wr.work_id AS id, r.isrc
            FROM resources r
            JOIN work_resources wr ON wr.resource_id = r.id
            WHERE r.isrc IN ({placeholders})
        ), party_shares AS (
            SELECT
                s.party_id,
                COUNT(DISTINCT s.work_id) AS num_works,
                SUM(s.share) AS total_shares
            FROM shares s
            WHERE s.work_id IN (SELECT id FROM matched_works)
            GROUP BY s.party_id
        ), party_isrcs AS (
            SELECT party_id, group_concat(isrc, ', ') AS isrcs
            FROM (
                SELECT DISTINCT s.party_id, mw.isrc
                FROM shares s
                JOIN matched_works mw ON mw.id = s.work_id
            )
            GROUP BY party_id
        )
        SELECT
            p.id, p.first_name, p.full_name, p.last_name, p.ipi, p.pro, p.role,
            ps.num_works,
            ps.total_shares,
            pi.isrcs
        FROM party_shares ps
        JOIN parties p ON p.id = ps.party_id
        JOIN party_isrcs pi ON pi.party_id = ps.party_id
        ORDER BY ps.total_shares DESC
        "
    );

    let mut stmt = db.prepare(&q)?;
    let rows = stmt.query_map(params_from_iter(isrcs.iter()), |row| {
        let (first, last) = {
            let a: Option<String> = row.get(1)?;
            let b: String = row.get(3)?;
            if (a.is_none() || a == Some("".into())) && b.is_empty() {
                (None, row.get(2)?)
            } else {
                (a, b)
            }
        };

        let num_works: usize = row.get::<_, i64>(7)? as usize;
        let total_shares: f32 = row.get(8)?;
        let avg_share = if num_works == 0 {
            0.0
        } else {
            total_shares / (num_works as f32)
        };
        Ok(PartyMeta {
            info: Party {
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
            },
            isrcs: row
                .get::<_, String>(9)?
                .split(", ")
                .map(|x| x.to_string())
                .collect(),
            num_works,
            total_shares,
            avg_share,
        })
    })?;

    rows.collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::{
        test_util::{assert_runs, empty_db},
        types::PartyInput,
    };
    use musicmeta::ipi::IpiNameNum;

    #[test]
    fn get_party_runs() {
        assert_runs(get_party(&empty_db(), 123));
    }

    #[test]
    fn get_party_collaborators_runs() {
        let db = empty_db();
        for flag in [false, true] {
            assert_runs(get_party_chain_of_title(&db, 123, flag));
        }
    }

    #[test]
    fn search_parties_runs() {
        let db = empty_db();
        assert_runs(search_parties(&db, PartyInput::Name("Name".into())));
        assert_runs(search_parties(
            &db,
            PartyInput::Ipi(IpiNameNum::new(1051977352).unwrap()),
        ));
    }

    #[test]
    fn parties_by_market_share_runs() {
        assert_runs(parties_by_market_share(&empty_db(), &["123".to_string()]));
    }
}
