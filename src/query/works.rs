use std::collections::HashMap;

use musicmeta::tis::society::TisSocietyCode;
use rusqlite::{ToSql, params, params_from_iter};

use crate::query::types::{PartyInfo, Release, Share, WorkInfo};

pub fn get_party_works(
    db: &rusqlite::Connection,
    party_id: u64,
    is_deep: bool,
) -> Result<Vec<WorkInfo>, rusqlite::Error> {
    let limit = 200;
    let q = "
        SELECT wk.id
        FROM shares s
        JOIN works wk on wk.id = s.work_id
        WHERE s.party_id = ?1
        LIMIT ?2
        ";
    let mut stmt = db.prepare(q)?;
    let work_ids = stmt
        .query_map(params![party_id as i64, limit], |row| {
            row.get::<_, String>(0)
        })?
        .collect::<Result<Vec<_>, _>>()?;

    get_works(db, work_ids.as_slice(), is_deep)
}

pub fn get_works<T: AsRef<str> + ToSql>(
    db: &rusqlite::Connection,
    work_ids: &[T],
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
          COALESCE(
              json_group_array(
                  CASE
                      WHEN r.id IS NOT NULL THEN json_object(
                        'id', r.id,
                        'title', r.title,
                        'artist_name', r.artist_name,
                        'label_name', r.label_name,
                        'distro_name', r.distro_name,
                        'isrc', rs.isrc,
                        'release_date', strftime('%Y-%m-%d', r.release_date, 'unixepoch')
                      )
                  END
              ),
              '[]'
          ) AS releases
        FROM works wk
        LEFT JOIN work_resources wr ON wr.work_id = wk.id
        LEFT JOIN resources rs ON rs.id = wr.resource_id
        LEFT JOIN releases r ON r.id = rs.release_id
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
            releases: row
                .get::<_, String>(5)
                .ok()
                .and_then(|x| serde_json::from_str::<Vec<Release>>(&x).ok())
                .unwrap_or_default(),
        };
        Ok((w.id.clone(), w))
    })?;

    let mut works_by_id = works_iter.collect::<Result<HashMap<_, _>, _>>()?;

    if is_deep {
        let work_ids: Vec<_> = works_by_id.keys().cloned().collect();
        let mut parties_by_work = get_works_parties(db, work_ids.as_slice())?;
        for (id, work) in works_by_id.iter_mut() {
            work.parties = parties_by_work.remove(id).unwrap_or_default();
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
                ipi: row.get::<_, i64>(3).ok().map(|x| x as u64),
                pro: row
                    .get::<_, i32>(4)
                    .ok()
                    .and_then(|x| TisSocietyCode::try_from(x as u16).ok()),
                shares: row
                    .get::<_, String>(5)
                    .ok()
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
          wk.in_dispute
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
          wk.in_dispute
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
