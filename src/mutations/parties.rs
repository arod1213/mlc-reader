use libsql::{Connection, params};
use musicmeta::{ipi::IpiNameNum, tis::society::TisSocietyCode};

use crate::types::{Party, WriterSearch};

pub async fn search_party(
    conn: &Connection,
    input: WriterSearch,
) -> Result<Vec<Party>, libsql::Error> {
    let sql = "
    SELECT
        full_name,
        ipi,
        pro
    FROM parties
    WHERE 
        (?1 IS NULL OR ipi = ?1)
    AND
        (?2 IS NULL OR full_name LIKE '%' || ?1 || '%')
    LIMIT 50
    ";
    let stmt = conn.prepare(sql).await?;
    let ipi = match input {
        WriterSearch::Ipi(x) => Some(x.0 as i64),
        _ => None,
    };
    let name = match input {
        WriterSearch::Name(x) => Some(x),
        _ => None,
    };
    let mut rows = stmt.query(params!(ipi, name)).await?;
    let mut parties = vec![];
    while let Some(row) = rows.next().await? {
        let p = Party {
            full_name: row.get(0)?,
            ipi_name_num: row.get::<u64>(1).map(IpiNameNum).ok(),
            pro: TisSocietyCode::try_from(row.get::<u64>(2)? as u16).ok(),
        };
        parties.push(p);
    }
    Ok(parties)
}
// pub fn search_writer_relations(&self, party_id: i64) {}
// pub fn search_publisher_relations(&self, party_id: i64) {}
// pub fn audit_party_catalog(&self, party_id: i64) {}
// }
