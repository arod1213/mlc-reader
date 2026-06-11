use clap::{Parser, ValueEnum};
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
    for line in r.lines() {
        let Ok(line) = line else {
            continue;
        };
        let Ok(update) = serde_json::from_str::<Update>(&line) else {
            continue;
        };
        update_publisher_writers(db, update.id, update.pro)
            .await
            .unwrap();
    }
}

async fn db_connect(is_local: bool) -> Result<libsql::Database, libsql::Error> {
    match is_local {
        true => {
            let url = env::var("DB_URL").expect("missing DB_URL");
            Builder::new_local(url).build().await
        }
        false => {
            let url = env::var("DB_URL").expect("missing DB_URL");
            let token = env::var("DB_TOKEN").expect("missing DB_TOKEN");
            Builder::new_remote(url, token).build().await
        }
    }
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    let db = db_connect(true).await.unwrap();

    let local_db = "mlc.db";
    let args = cli::Args::parse();
    match args.command {
        cli::Command::Migrate {} => migrate_from_bwarm_dump(local_db),
        cli::Command::Modify {} => {
            let conn = sqlite::open(local_db).unwrap();
            migrate_add_ons(&conn).unwrap();
        }
        cli::Command::Enrich { role } => {
            let conn = sqlite::open(local_db).unwrap();
            match role {
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
        cli::Command::Update {} => {
            let file = std::fs::File::open("update2.txt").unwrap();
            let mut reader = BufReader::new(file);
            handle_update(&mut reader, &db).await
        }
    }
}
