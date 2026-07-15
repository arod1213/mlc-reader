use clap::{Parser, ValueEnum};
use dotenv::dotenv;
use libsql::Builder;
use mlc_reader::migration::utils::disable_fk;
use mlc_reader::mutations::relations;
use mlc_reader::mutations::works::{self, WorkSearchParams};
use mlc_reader::{migration, server::Credential, update_pro_affiliations};
use musicmeta::ipi::IpiNameNum;
use serde::Deserialize;
use std::path::PathBuf;
use std::{env, io::BufReader};

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
    let db_url = env::var("DB_URL").expect("missing DB_URL");
    let is_local = db_url.starts_with("file:");
    let db = open_db(&db_url, is_local).await.unwrap();
    let conn = db.connect().unwrap();

    let args = Args::parse();
    match args.command {
        Command::GetWork { id } => {
            let res = works::get_works(&conn, &[id]).await.unwrap();
            dbg!(res);
        }
        Command::Relation { id } => {
            let res = relations::get_writer_collaborators(&conn, id, 0)
                .await
                .unwrap();
            dbg!(res);
        }
        Command::FindWork { artist, name } => {
            let q = WorkSearchParams {
                title: name,
                artist,
                party_ipi: Some(IpiNameNum(1051977352)),
                offset: 0,
                limit: 10,
                ..WorkSearchParams::default()
            };
            let res = works::search_works(&conn, q, true).await.unwrap();
            dbg!(res);
        }
        // save MLC BWARM TSV files onto disk
        Command::Save {} => {
            let cred = Credential {
                host: env::var("MLC_HOST").expect("missing MLC_HOST"),
                username: env::var("MLC_USER").expect("missing MLC_USER"),
                pw: env::var("MLC_PW").expect("missing MLC_PW"),
            };
            migration::save_remote_mlc_docs(&cred);
        }
        // save MLC BWARM TSV files into DB
        Command::Migrate { path } => {
            migration::migrate_from_bwarm_dump(&conn, &path).await;
        }
        // save MLC BWARM TSV files into DB
        Command::Modify {} => {
            migration::migrate_add_ons(&conn).await.unwrap();
        }
        // add relational tables and indexes in DB
        Command::Enrich { method } => {
            disable_fk(&conn).await.expect("failed to disable FKs");

            let tx = conn.transaction().await.unwrap();
            let res = async {
                match method {
                    EnrichMode::Writer => migration::enrich_writer_relations(&tx).await,
                    EnrichMode::Publisher => migration::enrich_publisher_relations(&tx).await,
                    EnrichMode::Role => migration::assign_roles(&tx).await,
                }
            }
            .await;
            match res {
                Ok(_) => {
                    eprintln!("insert succeeded");
                    tx.commit().await.unwrap()
                }
                Err(e) => {
                    eprintln!("failed to insert: {}", e);
                    tx.rollback().await.unwrap()
                }
            }
        }
        // update PRO affiliation for parties from JSONL doc
        Command::Update { path } => {
            let file = std::fs::File::open(path).unwrap();
            let mut reader = BufReader::new(file);
            update_pro_affiliations(&mut reader, &conn).await
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
    GetWork {
        #[arg(short, long)]
        id: String,
    },
    FindWork {
        #[arg(short, long)]
        name: Option<String>,
        #[arg(short, long)]
        artist: Option<String>,
    },
    Relation {
        #[arg(short, long)]
        id: i64,
    },
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
