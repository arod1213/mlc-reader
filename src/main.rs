use clap::{Parser, ValueEnum};
use dotenv::dotenv;
use mlc_reader::{
    additional::{
        create::assign_roles,
        local::{enrich_publisher_relations, enrich_writer_relations},
        migrate_add_ons,
    },
    handle_update,
    mutations::migrate::migrate_from_bwarm_dump,
    open_db, save_remote_mlc_docs,
    server::Credential,
};
use serde::Deserialize;
use std::path::PathBuf;
use std::{env, io::BufReader};

#[tokio::main]
async fn main() {
    dotenv().ok();
    let db_url = env::var("DB_URL").expect("missing DB_URL");
    let is_local = db_url.starts_with("file:");
    let db = open_db(&db_url, is_local).await.unwrap();
    let conn = db.connect().unwrap();

    let args = Args::parse();
    match args.command {
        // save MLC BWARM TSV files onto disk
        Command::Save {} => {
            let cred = Credential {
                host: env::var("MLC_HOST").expect("missing MLC_HOST"),
                username: env::var("MLC_USER").expect("missing MLC_USER"),
                pw: env::var("MLC_PW").expect("missing MLC_PW"),
            };
            save_remote_mlc_docs(&cred);
        }
        // save MLC BWARM TSV files into DB
        Command::Migrate { path } => {
            migrate_from_bwarm_dump(&conn, &path).await;
        }
        // save MLC BWARM TSV files into DB
        Command::Modify {} => {
            migrate_add_ons(&conn).await.unwrap();
        }
        // add relational tables and indexes in DB
        Command::Enrich { method } => {
            let tx = conn.transaction().await.unwrap();
            let res = async {
                match method {
                    EnrichMode::Writer => enrich_writer_relations(&tx).await,
                    EnrichMode::Publisher => enrich_publisher_relations(&tx).await,
                    EnrichMode::Role => assign_roles(&tx).await,
                }
            }
            .await;
            match res {
                Ok(_) => tx.commit().await.unwrap(),
                Err(_) => tx.rollback().await.unwrap(),
            }
        }
        // update PRO affiliation for parties from JSONL doc
        Command::Update { path } => {
            let file = std::fs::File::open(path).unwrap();
            let mut reader = BufReader::new(file);
            handle_update(&mut reader, &conn).await
        }
    }
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(clap::Subcommand, Debug)]
pub enum Command {
    Save {},
    Migrate {
        #[arg(short, long)]
        path: PathBuf,
    },
    Modify {},
    Enrich {
        #[arg(short, long)]
        method: EnrichMode,
    },
    Update {
        #[arg(short, long)]
        path: PathBuf,
    },
}

#[derive(Debug, Deserialize, Clone, ValueEnum)]
pub enum EnrichMode {
    Writer,
    Publisher,
    Role,
}
#[derive(Debug, Deserialize, Clone, ValueEnum)]
pub enum DiscoverMode {
    Writer,
}
