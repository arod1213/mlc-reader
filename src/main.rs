use clap::Parser;
use dotenv::dotenv;
use mlc_reader::migration::trim_db;
use mlc_reader::migration::utils::disable_fk;
use mlc_reader::{migration, server::Credential, update_pro_affiliations};
use std::error::Error;
use std::path::PathBuf;
use std::{env, io::BufReader};
use strum::{Display, EnumString};

struct JobConfig {
    out_dir: PathBuf,
    update_path: PathBuf,
    cred: Credential,
}

#[derive(EnumString, Display, Clone, PartialEq, PartialOrd)]
#[repr(u8)]
#[strum(serialize_all = "snake_case")]
enum RunStep {
    SaveDocs = 10,
    Schema = 20,
    Trim = 30,
    IndexSearch = 40,
    EnrichWriter = 50,
    EnrichPublisher = 60,
    EnrichRoles = 70,
    EnrichShares = 80,
    EnrichWorks = 90,
    UpdatePros = 100,
}

type RunFn<'a> = &'a dyn Fn(&mut rusqlite::Connection) -> Result<(), Box<dyn Error>>;
struct RunStepConfig<'a> {
    f: RunFn<'a>,
    step: RunStep,
}

fn nightjob(
    conn: &mut rusqlite::Connection,
    config: JobConfig,
    resume_step: Option<RunStep>,
) -> Result<(), Box<dyn Error>> {
    let steps = [
        RunStepConfig {
            step: RunStep::SaveDocs,
            f: &|_| migration::save_remote_mlc_docs(&config.cred, &config.out_dir),
        },
        RunStepConfig {
            step: RunStep::Schema,
            f: &|db| {
                migration::migrate_from_bwarm_dump(db, &config.out_dir)
                    .map_err(|e| e as Box<dyn Error>)
            },
        },
        RunStepConfig {
            step: RunStep::Trim,
            f: &|db| {
                trim_db(db, true)?;
                Ok(())
            },
        },
        RunStepConfig {
            step: RunStep::IndexSearch,
            f: &|db| Ok(migration::create_search_tables_indexes(db)?),
        },
        RunStepConfig {
            step: RunStep::EnrichWriter,
            f: &|db| {
                disable_fk(db)?;
                let tx = db.transaction()?;
                migration::enrich_writer_relations(&tx)?;
                Ok(tx.commit()?)
            },
        },
        RunStepConfig {
            step: RunStep::EnrichPublisher,
            f: &|db| {
                disable_fk(db)?;
                let tx = db.transaction()?;
                migration::enrich_publisher_relations(&tx)?;
                Ok(tx.commit()?)
            },
        },
        RunStepConfig {
            step: RunStep::EnrichRoles,
            f: &|db| {
                disable_fk(db)?;
                let tx = db.transaction()?;
                migration::assign_roles(&tx)?;
                Ok(tx.commit()?)
            },
        },
        RunStepConfig {
            step: RunStep::EnrichShares,
            f: &|db| {
                disable_fk(db)?;
                let tx = db.transaction()?;
                migration::add_party_stats(&tx)?;
                migration::enrich_work_totals(&tx)?;
                Ok(tx.commit()?)
            },
        },
        RunStepConfig {
            step: RunStep::UpdatePros,
            f: &|db| {
                let file = std::fs::File::open(&config.update_path)?;
                let mut reader = BufReader::new(file);
                Ok(update_pro_affiliations(&mut reader, db)?)
            },
        },
    ];
    for step in steps {
        if resume_step.as_ref().is_some_and(|r| *r > step.step) {
            continue;
        }

        log::info!("Running {}", step.step);
        (step.f)(conn)?;
    }

    Ok(())
}

fn main() {
    dotenv().ok();
    env_logger::init();

    let cred = Credential {
        host: env::var("MLC_HOST").expect("missing MLC_HOST"),
        username: env::var("MLC_USER").expect("missing MLC_USER"),
        public_key: env::var("MLC_PUBLIC_KEY").unwrap().into(),
        private_key: env::var("MLC_PRIVATE_KEY").unwrap().into(),
    };
    let out_dir: PathBuf = env::var("OUT_DIR").expect("missing OUT_DIR").into();
    let update_path: PathBuf = env::var("UPDATE_PATH").expect("missing UPDATE_PATH").into();

    let config = JobConfig {
        out_dir,
        update_path,
        cred,
    };

    let db_url = env::var("MLC_DB_URL").expect("missing MLC_DB_URL");
    let mut conn = rusqlite::Connection::open(&db_url).expect("failed to connect to db");

    let args = Args::parse();
    match args.command {
        Command::NightJob { resume_step } => nightjob(&mut conn, config, resume_step).unwrap(),
    }
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(clap::Subcommand)]
enum Command {
    NightJob {
        #[arg(short, long)]
        resume_step: Option<RunStep>,
    },
}
