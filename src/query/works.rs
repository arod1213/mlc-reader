use std::collections::HashMap;

use chrono::NaiveDate;
use musicmeta::tis::society::TisSocietyCode;
use rusqlite::{ToSql, params, params_from_iter};
use serde::Deserialize;

use crate::query::types::{PartyInfo, Release, Share, WorkInfo};

fn default_limit() -> usize {
    200
}
#[derive(Deserialize)]
pub struct PartyWorksQuery {
    #[serde(default)]
    pub with_id: Option<u64>,

    pub in_dispute: Option<bool>,

    #[serde(default = "default_limit")]
    pub limit: usize,

    #[serde(default)]
    pub page: usize,

    #[serde(default)]
    pub is_deep: bool,
}

pub fn get_party_works(
    db: &rusqlite::Connection,
    party_id: u64,
    params: PartyWorksQuery,
) -> Result<Vec<WorkInfo>, rusqlite::Error> {
    let limit = params.limit.min(200);
    let offset = params.limit * params.page;

    let in_dispute_clause = if params.in_dispute.is_some() {
        "in_dispute = ?3"
    } else {
        "?3 IS NULL"
    };

    let with_id_clause = if params.with_id.is_some() {
        "EXISTS (
                SELECT 1
                FROM shares
                WHERE party_id = ?2
                AND work_id = wk.id
            )"
    } else {
        "?2 IS NULL"
    };

    let q = format!(
        "
            SELECT wk.id
            FROM shares s
            JOIN works wk on wk.id = s.work_id
            WHERE s.party_id = ?1
            AND {with_id_clause}
            AND {in_dispute_clause}
            LIMIT ?4
            OFFSET ?5
        "
    );

    let mut stmt = db.prepare(&q)?;
    let work_ids = stmt
        .query_map(
            params![
                party_id as i64,
                params.with_id.map(|x| x as i64),
                params.in_dispute,
                limit as i64,
                offset as i64
            ],
            |row| row.get::<_, String>(0),
        )?
        .collect::<Result<Vec<_>, _>>()?;

    get_works(db, work_ids.as_slice(), 1_000, params.is_deep)
}

fn get_releases<T: AsRef<str> + ToSql>(
    db: &rusqlite::Connection,
    work_ids: &[T],
    limit: usize,
) -> Result<HashMap<String, Vec<Release>>, rusqlite::Error> {
    let placeholders = vec!["?"; work_ids.len()].join(", ");
    let q = format!(
        "SELECT
            wk.id,
            r.id,
            r.title,
            r.artist_name,
            r.label_name,
            r.distro_name,
            rs.isrc,
            strftime('%Y-%m-%d', r.release_date, 'unixepoch')
        FROM releases r
        JOIN resources rs on rs.release_id = r.id
        JOIN work_resources wr on wr.resource_id = rs.id
        JOIN works wk on wk.id = wr.work_id
        WHERE wk.id IN ({placeholders})
        AND r.title <> ''
        AND r.artist_name <> ''
        ORDER BY r.artist_name ASC
        LIMIT ?
        "
    );

    let mut stmt = db.prepare(&q)?;
    let limit = limit as i64;
    let p = params_from_iter(
        work_ids
            .iter()
            .map(|id| id as &dyn ToSql)
            .chain(std::iter::once(&limit as &dyn ToSql)),
    );

    let rows = stmt.query_map(p, |row| {
        let work_id: String = row.get(0)?;
        let r = Release {
            id: row.get(1)?,
            title: row.get(2)?,
            artist_name: row.get(3)?,
            label_name: row.get(4)?,
            distro_name: row.get(5)?,
            isrc: row.get(6)?,
            release_date: row
                .get::<_, Option<String>>(7)?
                .and_then(|x| NaiveDate::parse_from_str(&x, "%Y-%m-%d").ok()),
        };
        Ok((work_id, r))
    })?;

    let release_by_work = rows.collect::<Result<Vec<_>, _>>()?;
    let mut map: HashMap<String, Vec<Release>> = HashMap::new();
    for (id, release) in release_by_work {
        match map.get_mut(&id) {
            Some(x) => x.push(release),
            None => {
                map.insert(id, vec![release]);
            }
        }
    }
    Ok(map)
}

pub fn get_works<T: AsRef<str> + ToSql>(
    db: &rusqlite::Connection,
    work_ids: &[T],
    release_limit: usize,
    is_deep: bool,
) -> Result<Vec<WorkInfo>, rusqlite::Error> {
    let placeholders = vec!["?"; work_ids.len()].join(", ");
    let q = format!(
        "
        SELECT
          wk.id,
          wk.title,
          wk.duration_ms,
          wk.iswc,
          wk.in_dispute,
          wk.total_shares
        FROM works wk
        WHERE wk.id IN ({placeholders})
        GROUP BY wk.id, wk.title, wk.duration_ms, wk.iswc, wk.in_dispute
        "
    );

    let mut stmt = db.prepare(&q)?;
    let works_iter = stmt.query_map(params_from_iter(work_ids.iter()), |row| {
        let w = WorkInfo {
            id: row.get(0)?,
            title: row.get(1)?,
            duration_ms: row.get(2)?,
            iswc: row.get::<_, Option<String>>(3)?,
            in_dispute: row.get(4)?,
            parties: vec![],
            total_shares: row.get(5)?,
            releases: vec![],
        };
        Ok((w.id.clone(), w))
    })?;

    let mut works_by_id = works_iter.collect::<Result<HashMap<_, _>, _>>()?;

    if is_deep {
        let work_ids: Vec<_> = works_by_id.keys().cloned().collect();
        let mut releases_by_work = get_releases(db, &work_ids, release_limit)?;
        let mut parties_by_work = get_works_parties(db, &work_ids)?;
        for (id, work) in works_by_id.iter_mut() {
            work.parties = parties_by_work.remove(id).unwrap_or_default();
            work.releases = releases_by_work.remove(id).unwrap_or_default();
        }
    }
    Ok(works_by_id.into_values().collect())
}

pub fn get_works_parties<T: ToString + ToSql>(
    db: &rusqlite::Connection,
    work_ids: &[T],
) -> Result<HashMap<String, Vec<PartyInfo>>, rusqlite::Error> {
    if work_ids.is_empty() {
        return Ok(HashMap::new());
    }

    let placeholders = vec!["?"; work_ids.len()].join(", ");
    let q = format!(
        "
        SELECT 
            s.work_id as work_id, 
            p.id as party_id,
            p.full_name,
            p.ipi,
            p.pro,
            COALESCE(
                json_group_array(
                    CASE
                        WHEN s.id IS NOT NULL THEN json_object(
                            'id', s.id,
                            'party_id', p.id,
                            'role', s.role,
                            'rights_type', s.rights_type,
                            'share', s.share,
                            'preceding_id', s.preceding_id
                        )
                    END
                ),
                '[]'
            ) as party_shares
        FROM shares s
        JOIN parties p ON p.id = s.party_id
        WHERE s.work_id IN ({placeholders})
        GROUP BY s.work_id, p.id, p.full_name, p.ipi, p.pro"
    );

    let mut stmt = db.prepare(&q)?;
    let parties = stmt
        .query_map(params_from_iter(work_ids.iter()), |row| {
            let work_id: String = row.get(0)?;
            let p = PartyInfo {
                id: row.get::<_, i64>(1)? as u64,
                full_name: row.get(2)?,
                ipi: row.get::<_, Option<i64>>(3)?.map(|x| x as u64),
                pro: row
                    .get::<_, Option<i32>>(4)?
                    .and_then(|x| TisSocietyCode::try_from(x as u16).ok()),
                shares: row
                    .get::<_, Option<String>>(5)?
                    .and_then(|s| serde_json::from_str::<Vec<Share>>(&s).ok())
                    .unwrap_or_default(),
            };
            Ok((work_id, p))
        })?
        .collect::<Result<Vec<_>, _>>()?;

    let mut v: HashMap<String, Vec<PartyInfo>> = HashMap::new();
    for (work_id, party) in parties {
        match v.get_mut(&work_id) {
            Some(entry) => entry.push(party),
            None => {
                v.insert(work_id, vec![party]);
            }
        }
    }
    Ok(v)
}

pub fn search_works_by_isrcs<T: AsRef<str> + ToSql>(
    db: &rusqlite::Connection,
    isrcs: &[T],
) -> Result<HashMap<String, Vec<WorkInfo>>, rusqlite::Error> {
    let placeholders = vec!["?"; isrcs.len()].join(", ");
    let q = format!(
        "
        WITH matched_resources AS (
            SELECT id, isrc
            FROM resources
            WHERE isrc IN ({placeholders})
        )
        SELECT
          r.isrc,
          wk.id,
          wk.title,
          wk.duration_ms,
          wk.iswc,
          wk.in_dispute,
          wk.total_shares
        FROM matched_resources r
        JOIN work_resources wr ON wr.resource_id = r.id
        JOIN works wk on wk.id = wr.work_id
        "
    );

    let mut stmt = db.prepare(&q)?;
    let rows = stmt.query_map(params_from_iter(isrcs.iter()), |row| {
        let isrc: String = row.get(0)?;
        let w = WorkInfo {
            id: row.get(1)?,
            title: row.get(2)?,
            duration_ms: row.get(3)?,
            iswc: row.get::<_, Option<String>>(4)?,
            in_dispute: row.get(5)?,
            total_shares: row.get(6)?,
            parties: vec![],
            releases: vec![],
        };
        Ok((isrc, w))
    })?;

    let mut works_by_isrc: HashMap<String, Vec<WorkInfo>> = HashMap::new();
    for row in rows {
        let (isrc, work) = row?;
        works_by_isrc.entry(isrc).or_default().push(work);
    }
    Ok(works_by_isrc)
}

pub fn search_works_by_iswcs<T: AsRef<str> + ToSql>(
    db: &rusqlite::Connection,
    iswcs: &[T],
) -> Result<HashMap<String, Vec<WorkInfo>>, rusqlite::Error> {
    let placeholders = vec!["?"; iswcs.len()].join(", ");
    let q = format!(
        "
        SELECT
          wk.id,
          wk.title,
          wk.duration_ms,
          wk.iswc,
          wk.in_dispute,
          wk.total_shares
        FROM matched_resources r
        JOIN work_resources wr ON wr.resource_id = r.id
        FROM works wk
        WHERE wk.iswc IN ({placeholders})
        "
    );

    let mut stmt = db.prepare(&q)?;
    let rows = stmt.query_map(params_from_iter(iswcs.iter()), |row| {
        let iswc: String = row.get(3)?;
        let w = WorkInfo {
            id: row.get(0)?,
            title: row.get(1)?,
            duration_ms: row.get(2)?,
            iswc: Some(iswc.clone()),
            in_dispute: row.get(4)?,
            total_shares: row.get(5)?,
            parties: vec![],
            releases: vec![],
        };
        Ok((iswc, w))
    })?;

    let mut works_by_iswc: HashMap<String, Vec<WorkInfo>> = HashMap::new();
    for row in rows {
        let (iswc, work) = row?;
        works_by_iswc.entry(iswc).or_default().push(work);
    }
    Ok(works_by_iswc)
}

pub fn search_works_by_title(
    db: &rusqlite::Connection,
    artist: Option<&str>,
    title: &str,
) -> Result<Vec<WorkInfo>, rusqlite::Error> {
    let artist_condition = if artist.is_some() {
        "
        JOIN work_resources wr on wr.work_id = wk.id
        JOIN resources rs on rs.id = wr.resource_id
        JOIN releases r on r.id = rs.release_id
        WHERE r.artist_name = ?2
        "
    } else {
        "WHERE ?2 is null"
    };
    let q = format!(
        "
        SELECT
          wk.id,
          wk.title,
          wk.duration_ms,
          wk.iswc,
          wk.in_dispute,
          wk.total_shares
        FROM works wk
        {artist_condition}
        AND wk.title LIKE ?1
        "
    );

    let mut stmt = db.prepare(&q)?;
    let rows = stmt.query_map(params![title, artist], |row| {
        Ok(WorkInfo {
            id: row.get(0)?,
            title: row.get(1)?,
            duration_ms: row.get(2)?,
            iswc: row.get::<_, Option<String>>(3)?,
            in_dispute: row.get(4)?,
            total_shares: row.get(5)?,
            parties: vec![],
            releases: vec![],
        })
    })?;

    rows.collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::test_util::{assert_runs, empty_db};

    #[test]
    fn get_party_works_runs() {
        for flag in [true, false] {
            let q = PartyWorksQuery {
                with_id: Some(123),
                in_dispute: Some(true),
                limit: 10,
                page: 0,
                is_deep: flag,
            };
            assert_runs(get_party_works(&empty_db(), 123, q));

            let q = PartyWorksQuery {
                with_id: None,
                in_dispute: None,
                limit: 10,
                page: 0,
                is_deep: flag,
            };
            assert_runs(get_party_works(&empty_db(), 123, q));
        }
    }
}
