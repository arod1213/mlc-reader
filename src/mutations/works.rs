use libsql::{Connection, params};
use musicmeta::iswc::Iswc;
use serde::{Deserialize, Serialize};

pub enum WorkSearch {
    Iswc(Iswc),
    Track { title: String, artist: String },
}

#[derive(Debug, Default, Serialize)]
pub struct WorkInfo {
    pub id: String,
    pub title: String,
    pub duration_ms: Option<f64>,
    pub iswc: Option<Iswc>,
    pub in_dispute: bool,
    pub releases: Vec<Release>,
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
            releases: row
                .get::<String>(5)
                .ok()
                .and_then(|x| serde_json::from_str::<Vec<Release>>(&x).ok())
                .unwrap_or_default(),
        });
    }
    Ok(v)
}
