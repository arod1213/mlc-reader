use clap::Parser;
use cwr::models::society::SocietyCode;
use dotenv::dotenv;
use libsql::{Builder, Database};
use serde::Deserialize;
use std::{
    env,
    io::{BufRead, BufReader},
};

use crate::{
    additional::migrate_add_ons, save::migrate_from_bwarm_dump, update::update_publisher_writers,
};

mod additional;
pub mod bwarm;
mod cli;
pub mod commands;
pub mod save;
pub mod update;

#[derive(Debug, Deserialize)]
pub struct Update {
    pub id: i64,
    pub pro: SocietyCode,
}

async fn handle_update<R: BufRead>(r: &mut R, db: &Database) {
    let conn = db.connect().unwrap();
    for line in r.lines() {
        let Ok(line) = line else {
            continue;
        };
        let Ok(update) = serde_json::from_str::<Update>(&line) else {
            continue;
        };
        update_publisher_writers(&conn, update.id, update.pro)
            .await
            .unwrap();
    }
}

async fn open_db(url: &str, is_local: bool) -> Result<libsql::Database, libsql::Error> {
    match is_local {
        true => Builder::new_local(url).build().await,
        false => {
            let token = env::var("DB_TOKEN").expect("missing DB_TOKEN");
            Builder::new_remote(url.into(), token).build().await
        }
    }
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    let is_local = env::var("DB_MODE") == Ok("LOCAL".into());
    let db_url = env::var("DB_URL").expect("missing DB_URL");
    let db = open_db(&db_url, is_local).await.unwrap();

    let args = cli::Args::parse();
    match args.command {
        cli::Command::Migrate {} => {
            let conn = db.connect().unwrap();
            migrate_from_bwarm_dump(&conn).await;
        }
        cli::Command::Modify {} => {
            let conn = db.connect().unwrap();
            migrate_add_ons(&conn).await.unwrap();
        }
        cli::Command::Discover { method } => {
            //
            match method {
                cli::DiscoverMode::Writer => todo!(),
            }
        }
        cli::Command::Enrich { method } => {
            let conn = sqlite::open(&db_url).unwrap();
            match method {
                cli::EnrichMode::Writer => {
                    additional::local::wrap_tx(&conn, additional::local::enrich_writer_relations)
                        .unwrap()
                }

                cli::EnrichMode::Publisher => {
                    additional::local::wrap_tx(&conn, additional::local::enrich_publisher_relations)
                        .unwrap()
                }
                cli::EnrichMode::Role => {
                    additional::local::wrap_tx(&conn, additional::create::assign_roles).unwrap()
                }
            }
        }
        cli::Command::Update { path } => {
            let file = std::fs::File::open(path).unwrap();
            let mut reader = BufReader::new(file);
            handle_update(&mut reader, &db).await
        }
    }
}
