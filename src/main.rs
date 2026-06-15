mod cli;

use clap::Parser;
use cli::Command::*;
use dotenv::dotenv;
use mlc_reader::{
    additional::{
        create::assign_roles,
        local::{enrich_publisher_relations, enrich_writer_relations},
        migrate_add_ons,
    },
    handle_update, open_db,
    save::migrate_from_bwarm_dump,
    save_remote_mlc_docs,
    server::Credential,
};
use std::{env, io::BufReader};

#[tokio::main]
async fn main() {
    dotenv().ok();
    let db_url = env::var("DB_URL").expect("missing DB_URL");
    let is_local = db_url.starts_with("file:");
    let db = open_db(&db_url, is_local).await.unwrap();
    let conn = db.connect().unwrap();

    let args = cli::Args::parse();
    match args.command {
        Save {} => {
            let cred = Credential {
                host: env::var("MLC_HOST").expect("missing MLC_HOST"),
                username: env::var("MLC_USER").expect("missing MLC_USER"),
                pw: env::var("MLC_PW").expect("missing MLC_PW"),
            };
            save_remote_mlc_docs(&cred);
        }
        Migrate { path } => {
            migrate_from_bwarm_dump(&conn, &path).await;
        }
        Modify {} => {
            migrate_add_ons(&conn).await.unwrap();
        }
        Enrich { method } => {
            let tx = conn.transaction().await.unwrap();
            let res = async {
                match method {
                    cli::EnrichMode::Writer => enrich_writer_relations(&tx).await,
                    cli::EnrichMode::Publisher => enrich_publisher_relations(&tx).await,
                    cli::EnrichMode::Role => assign_roles(&tx).await,
                }
            }
            .await;
            match res {
                Ok(_) => tx.commit().await.unwrap(),
                Err(_) => tx.rollback().await.unwrap(),
            }
        }
        Update { path } => {
            let file = std::fs::File::open(path).unwrap();
            let mut reader = BufReader::new(file);
            handle_update(&mut reader, &conn).await
        }
        Discover { method } => {
            //
            match method {
                cli::DiscoverMode::Writer => todo!(),
            }
        }
    }
}
