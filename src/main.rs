use clap::{Parser, ValueEnum};
use dotenv::dotenv;
use mlc_reader::migration::trim_db;
use mlc_reader::migration::utils::disable_fk;
use mlc_reader::types::Role;
use mlc_reader::{migration, server::Credential, update_pro_affiliations};
use musicmeta::ipi::IpiNameNum;
use musicmeta::iswc::Iswc;
use serde::Deserialize;
use std::path::PathBuf;
use std::{env, io::BufReader};

fn main() {
    dotenv().ok();
    env_logger::init();

    let db_url = env::var("MLC_DB_URL").expect("missing MLC_DB_URL");
    let mut conn = rusqlite::Connection::open(&db_url).expect("failed to connect to db");

    let args = Args::parse();
    match args.command {
        Command::Talent {} => {
            // let res = top_unsigned_writers(&conn).unwrap();
            // dbg!(res);
        }
        Command::Relation { id } => {
            // let res = relations::get_writer_collaborators(&conn, id, 0)
            //
            //     .unwrap();
            // dbg!(res);
        }
        Command::PartySearch { input, role } => {
            // let res = search_parties(&conn, input.into(), role, 30).unwrap();
            // dbg!(res);
        }
        Command::Work { id } => {
            // let res = works::get_works(&conn, &[id]).unwrap();
            // dbg!(res);
        }
        Command::WorkSearch { mode } => {
            // let offset: usize = 0;
            // let limit: usize = 100;
            // let is_deep = false;
            // let res = match mode {
            //     SearchMode::Party { ipi } => {
            //         works::search_party_works(&conn, ipi, offset, limit, is_deep)
            //     }
            //     SearchMode::Record { isrc } => {
            //         let isrc = Isrc::from_str(&isrc).expect("invalid isrc");
            //         works::search_works_by_isrc(&conn, isrc, offset, limit, is_deep)
            //     }
            //     SearchMode::Work { iswc } => {
            //         works::search_works_by_iswc(&conn, iswc, offset, limit, is_deep)
            //     }
            // }
            // .unwrap();
            // dbg!(res);
        }
        // save MLC BWARM TSV files onto disk
        Command::Save { path } => {
            let cred = Credential {
                host: env::var("MLC_HOST").expect("missing MLC_HOST"),
                username: env::var("MLC_USER").expect("missing MLC_USER"),
                public_key: env::var("MLC_PUBLIC_KEY").unwrap().into(),
                private_key: env::var("MLC_PRIVATE_KEY").unwrap().into(),
            };
            migration::save_remote_mlc_docs(&cred, &path);
        }
        // save MLC BWARM TSV files into DB
        Command::Migrate { path } => {
            migration::migrate_from_bwarm_dump(&mut conn, &path);
        }
        // save MLC BWARM TSV files into DB
        Command::IndexSearch {} => {
            migration::create_search_tables_indexes(&conn).unwrap();
        }
        Command::IndexTrim {} => {
            migration::create_trim_shares_indexes(&conn).unwrap();
        }
        Command::Trim { vacuum } => {
            trim_db(&conn, vacuum).unwrap();
        }
        // add relational tables and indexes in DB
        Command::Enrich { method } => {
            disable_fk(&conn).expect("failed to disable FKs");

            let tx = conn.transaction().unwrap();
            let res = {
                match method {
                    EnrichMode::Writer => migration::enrich_writer_relations(&tx),
                    EnrichMode::Publisher => migration::enrich_publisher_relations(&tx),
                    EnrichMode::Role => migration::assign_roles(&tx),
                    EnrichMode::Share => migration::add_party_stats(&tx),
                }
            };
            match res {
                Ok(_) => {
                    log::info!("insert succeeded");
                    tx.commit().unwrap()
                }
                Err(e) => {
                    log::error!("failed to insert: {}", e);
                    tx.rollback().unwrap()
                }
            }
        }
        // update PRO affiliation for parties from JSONL doc
        Command::Update { path } => {
            let file = std::fs::File::open(path).unwrap();
            let mut reader = BufReader::new(file);
            update_pro_affiliations(&mut reader, &mut conn)
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
pub enum SearchMode {
    Party {
        #[arg(short, long)]
        ipi: IpiNameNum,
    },
    Record {
        #[arg(short, long)]
        isrc: String,
    },
    Work {
        #[arg(short, long)]
        iswc: Iswc,
    },
}

#[derive(clap::Subcommand, Debug)]
pub enum Command {
    Talent {},
    Work {
        #[arg(short, long)]
        id: String,
    },
    PartySearch {
        #[arg(short, long)]
        input: String,
        #[arg(short, long)]
        role: Option<Role>,
    },
    WorkSearch {
        #[command(subcommand)]
        mode: SearchMode,
    },
    Relation {
        #[arg(short, long)]
        id: i64,
    },
    Save {
        #[arg(short, long)]
        path: PathBuf,
    },
    Migrate {
        #[arg(short, long)]
        path: PathBuf,
    },
    IndexSearch {},
    IndexTrim {},
    Trim {
        #[arg(short, long)]
        vacuum: bool,
    },
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
    Share,
}
#[derive(Debug, Deserialize, Clone, ValueEnum)]
pub enum DiscoverMode {
    Writer,
}
