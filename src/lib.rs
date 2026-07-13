use libsql::Connection;
use musicmeta::tis::society::TisSocietyCode;
use serde::Deserialize;
use std::io::BufRead;

pub mod bwarm;
pub mod migration;
pub mod mutations;
pub mod server;
pub mod types;
pub mod validation;

#[derive(Debug, Deserialize)]
pub struct Update {
    pub id: i64,
    pub pro: TisSocietyCode,
}

/// requires JSONL input of type struct Update{}
pub async fn update_pro_affiliations<R: BufRead>(r: &mut R, conn: &Connection) {
    for line in r.lines() {
        let Ok(line) = line else {
            continue;
        };
        let Ok(update) = serde_json::from_str::<Update>(&line) else {
            continue;
        };
        mutations::society::update_publisher_writers(conn, update.id, update.pro)
            .await
            .unwrap();
    }
}
