use std::{env, io::stdin};

use crate::{save::migrate_and_save, update::update_publisher_writers};

pub mod bwarm;
pub mod save;
pub mod update;
use clap::{Parser, ValueEnum};
use dotenv::dotenv;
use libsql::Builder;
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone, ValueEnum)]
pub enum Command {
    Migrate,
    Update,
}

#[derive(Debug, Parser)]
pub struct Args {
    #[arg(short, long)]
    pub command: Command,
}

#[derive(Debug, Deserialize)]
pub struct Update {
    pub id: String,
    pub pro: u64,
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    let url = env::var("DB_URL").expect("missing DB_URL");
    let token = env::var("DB_TOKEN").expect("missing DB_TOKEN");
    let db = Builder::new_remote(url, token).build().await.unwrap();

    let args = Args::parse();
    match args.command {
        Command::Migrate => migrate_and_save(),
        Command::Update => {
            let updates: Vec<Update> = serde_json::from_reader(stdin()).unwrap();
            for update in updates {
                update_publisher_writers(&db, &update.id, update.pro as i64)
                    .await
                    .unwrap();
            }
        }
    }
}
