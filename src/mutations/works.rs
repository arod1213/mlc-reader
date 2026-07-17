use std::collections::HashMap;

use libsql::{Connection, params};
use musicmeta::{ipi::IpiNameNum, isrc::Isrc, iswc::Iswc, tis::society::TisSocietyCode};
use serde::{Deserialize, Serialize};

pub enum WorkSearch {
    Iswc(Iswc),
    Track { title: String, artist: String },
}

#[derive(Debug, Default, Serialize)]
pub struct PartyInfo {
    pub id: u64,
    pub full_name: String,
    pub ipi: Option<IpiNameNum>,
    pub pro: Option<TisSocietyCode>,
    pub shares: Vec<Share>,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Share {
    pub id: String,
    pub role: String,
    pub share_type: Option<String>,
    pub rights_type: Option<String>,
    pub share: f64,
    pub preceding_id: Option<String>,
    // pub territory: String,
}

#[derive(Debug, Default, Serialize)]
pub struct WorkInfo {
    pub id: String,
    pub title: String,
    pub duration_ms: Option<f64>,
    pub iswc: Option<Iswc>,
    pub in_dispute: bool,
    pub releases: Vec<Release>,
    pub parties: Vec<PartyInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Release {
    pub id: i64,
    pub title: String,
    pub artist_name: String,
    pub label_name: String,
    pub distro_name: String,
    pub isrc: Option<Isrc>,
}

#[derive(Debug, Default)]
pub struct WorkSearchParams {
    pub iswc: Option<Iswc>,
    pub isrc: Option<Isrc>,
    pub title: Option<String>,
    pub artist: Option<String>,
    pub party_ipi: Option<IpiNameNum>,
    pub limit: usize,
    pub offset: usize,
}

pub async fn search_works(
    conn: &Connection,
    q: WorkSearchParams,
    is_deep: bool,
) -> Result<Vec<WorkInfo>, libsql::Error> {
    let sql = "
        WITH matched_works AS (
          SELECT wk.id
          FROM works wk
          WHERE 1 = 1
            AND (?1 IS NULL OR wk.iswc = ?1)
            AND (?2 IS NULL OR wk.title = ?2)
            AND (
              ?3 IS NULL OR EXISTS (
                SELECT 1
                FROM work_resources wr
                JOIN resources rs ON rs.id = wr.resource_id
                JOIN releases r ON r.id = rs.release_id
                WHERE wr.work_id = wk.id
                  AND r.artist_name = ?3
              )
            )
            AND (
              ?4 IS NULL OR EXISTS (
                SELECT 1
                FROM work_resources wr
                JOIN resources rs ON rs.id = wr.resource_id
                WHERE wr.work_id = wk.id
                    AND rs.isrc = ?4
              )
            )
            AND (
              ?5 IS NULL OR EXISTS (
                SELECT 1
                FROM shares s
                JOIN parties p ON p.id = s.party_id
                WHERE s.work_id = wk.id
                  AND p.ipi = ?5
              )
            )
          LIMIT ?6 OFFSET ?7
        )
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
                          'isrc', rs.isrc
                      )
                  END
              ),
              '[]'
          ) AS releases
        FROM matched_works mw
        JOIN works wk ON wk.id = mw.id
        LEFT JOIN work_resources wr ON wr.work_id = wk.id
        LEFT JOIN resources rs ON rs.id = wr.resource_id
        LEFT JOIN releases r ON r.id = rs.release_id
        GROUP BY wk.id, wk.title, wk.duration_ms, wk.iswc, wk.in_dispute;";
    let mut rows = conn
        .query(
            sql,
            params!(
                q.iswc.map(|x| x.to_string()),
                q.title.map(|x| x.to_uppercase()),
                q.artist.map(|x| x.to_uppercase()),
                q.isrc.map(|x| x.to_string()),
                q.party_ipi.map(|x| x.0 as i64),
                q.limit as i64,
                (q.offset * q.limit) as i64,
            ),
        )
        .await?;
    let mut works_by_id: HashMap<String, WorkInfo> = HashMap::new();
    while let Some(row) = rows.next().await? {
        let work_id: String = row.get(0)?;
        works_by_id.insert(
            work_id.clone(),
            WorkInfo {
                id: work_id,
                title: row.get(1)?,
                duration_ms: row.get(2)?,
                iswc: row
                    .get::<Option<String>>(3)?
                    .and_then(|s| Iswc::try_from(s).ok()),
                in_dispute: row.get(4)?,
                parties: vec![],
                releases: row
                    .get::<String>(5)
                    .ok()
                    .and_then(|x| serde_json::from_str::<Vec<Release>>(&x).ok())
                    .unwrap_or_default(),
            },
        );
    }
    if is_deep {
        let work_ids: Vec<_> = works_by_id.keys().cloned().collect();
        let mut parties_by_work = get_works_parties(conn, work_ids.as_slice()).await?;
        for (id, work) in works_by_id.iter_mut() {
            work.parties = parties_by_work.remove(id).unwrap_or_default();
        }
    }
    Ok(works_by_id.into_values().collect())
}

pub async fn get_works(conn: &Connection, ids: &[String]) -> Result<Vec<WorkInfo>, libsql::Error> {
    if ids.is_empty() {
        return Ok(vec![]);
    }

    let ids_json = serde_json::json!(ids).to_string();
    let sql = "
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
                            'distro_name', r.distro_name
                        )
                    END
                ),
                '[]'
            ) as releases
        FROM works wk
        LEFT JOIN work_resources wr ON wr.work_id = wk.id
        LEFT JOIN resources rs ON rs.id = wr.resource_id
        LEFT JOIN releases r on r.id = rs.release_id
        WHERE wk.id IN (SELECT value FROM json_each(?1))
        GROUP BY wk.id, wk.title, wk.duration_ms, wk.iswc, wk.in_dispute
        LIMIT 15;";
    let mut rows = conn.query(sql, params!(ids_json)).await?;

    let mut parties_by_work = get_works_parties(conn, ids).await?;

    let mut v = vec![];
    while let Some(row) = rows.next().await? {
        let work_id: String = row.get(0)?;
        let parties = parties_by_work.remove(&work_id).unwrap_or_default();

        let w = WorkInfo {
            id: work_id,
            title: row.get(1)?,
            duration_ms: row.get(2)?,
            iswc: row
                .get::<Option<String>>(3)?
                .and_then(|s| Iswc::try_from(s).ok()),
            in_dispute: row.get(4)?,
            parties,
            releases: row
                .get::<String>(5)
                .ok()
                .and_then(|x| serde_json::from_str::<Vec<Release>>(&x).ok())
                .unwrap_or_default(),
        };
        v.push(w);
    }
    Ok(v)
}

pub async fn get_works_parties(
    conn: &Connection,
    ids: &[String],
) -> Result<HashMap<String, Vec<PartyInfo>>, libsql::Error> {
    if ids.is_empty() {
        return Ok(HashMap::new());
    }

    let ids_json = serde_json::json!(ids).to_string();
    let sql = "
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
                            'role', s.role,
                            'share_type', s.share_type,
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
        WHERE s.work_id IN (SELECT value FROM json_each(?1))
        GROUP BY s.work_id, p.id, p.full_name, p.ipi, p.pro";

    let mut rows = conn.query(sql, params!(ids_json)).await?;

    let mut v: HashMap<String, Vec<PartyInfo>> = HashMap::new();
    while let Some(row) = rows.next().await? {
        let work_id: String = row.get(0)?;
        let p = PartyInfo {
            id: row.get::<i64>(1)? as u64,
            full_name: row.get(2)?,
            ipi: row
                .get::<i64>(3)
                .ok()
                .and_then(|x| IpiNameNum::new(x as u64).ok()),
            pro: row
                .get::<i32>(4)
                .ok()
                .and_then(|x| TisSocietyCode::try_from(x as u16).ok()),
            shares: row
                .get::<String>(5)
                .ok()
                .and_then(|s| serde_json::from_str::<Vec<Share>>(&s).ok())
                .unwrap_or_default(),
        };
        match v.get_mut(&work_id) {
            Some(entry) => entry.push(p),
            None => {
                v.insert(work_id, vec![p]);
            }
        }
    }
    Ok(v)
}
