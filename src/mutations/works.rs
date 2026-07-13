use std::collections::HashMap;

use libsql::{Connection, params};
use musicmeta::{ipi::IpiNameNum, iswc::Iswc, tis::society::TisSocietyCode};
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
    pub share: Share,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Share {
    pub id: String,
    pub role: String,
    pub share_type: String,
    pub rights_type: String,
    pub share: f64,
    // pub territory: String,
    // pub preceding_id: String,
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
}

#[derive(Debug, Default)]
pub struct WorkSearchParams {
    pub iswc: Option<Iswc>,
    pub title: Option<String>,
    pub artist: Option<String>,
}

pub async fn search_works(
    conn: &Connection,
    q: WorkSearchParams,
) -> Result<Vec<WorkInfo>, libsql::Error> {
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
        WHERE (
            $1::text is NULL 
            OR wk.iswc = $1::text
        )
        AND (
            $2::text is NULL 
            OR wk.title LIKE '%' || $2::text || '%'
        )
        AND (
            $3::text is NULL 
            OR r.artist_name LIKE '%' || $3::text || '%'
        )
        GROUP BY wk.id, wk.title, wk.duration_ms, wk.iswc, wk.in_dispute
        LIMIT 15;";
    let mut rows = conn
        .query(
            sql,
            params!(
                q.iswc.map(|x| x.to_string()),
                q.title.map(|x| x.to_uppercase()),
                q.artist.map(|x| x.to_uppercase())
            ),
        )
        .await?;
    let mut v = vec![];
    while let Some(row) = rows.next().await? {
        v.push(WorkInfo {
            id: row.get(0)?,
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
        });
    }
    Ok(v)
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
            wk.id as work_id, 
            p.id as party_id,
            p.full_name,
            p.ipi,
            p.pro,
            s.id as share_id,
            s.role,
            s.share_type,
            s.rights_type,
            s.share
        FROM parties p
        JOIN shares s on s.party_id = p.id
        LEFT JOIN works wk on wk.id = s.work_id
        WHERE wk.id IN (SELECT value FROM json_each(?1))
        LIMIT 15;";

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
            share: Share {
                id: row.get(5)?,
                role: row.get(6)?,
                share_type: row.get(7)?,
                rights_type: row.get(8)?,
                share: row.get(9)?,
            },
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
