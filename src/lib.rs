use crate::{
    bwarm::types::{Party, Release, Share, Work},
    server::Credential,
};
use libsql::{Builder, Connection};
use musicmeta::tis::society::TisSocietyCode;
use serde::Deserialize;
use std::{env, io::BufRead};

pub mod additional;
pub mod bwarm;
pub mod mutations;
pub mod server;
pub mod types;

pub fn save_remote_mlc_docs(cred: &Credential) {
    let sftp = cred.open().unwrap();
    let dir = server::latest_dir(&sftp).expect("missing MLC dirs");
    server::save_doc::<Release>(&sftp, &dir).unwrap();
    server::save_doc::<Work>(&sftp, &dir).unwrap();
    server::save_doc::<Party>(&sftp, &dir).unwrap();
    server::save_doc::<Share>(&sftp, &dir).unwrap();
}

#[derive(Debug, Deserialize)]
pub struct Update {
    pub id: i64,
    pub pro: TisSocietyCode,
}

pub async fn handle_update<R: BufRead>(r: &mut R, conn: &Connection) {
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

pub async fn open_db(url: &str, is_local: bool) -> Result<libsql::Database, libsql::Error> {
    match is_local {
        true => Builder::new_local(url).build().await,
        false => {
            let token = env::var("DB_TOKEN").expect("missing DB_TOKEN");
            Builder::new_remote(url.into(), token).build().await
        }
    }
}
