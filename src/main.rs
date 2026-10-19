use clap::Parser;
use dotenv::dotenv;
use mlc_reader::migration::utils::disable_fk;
use mlc_reader::migration::{setup_trim_indexes, trim_db};
use mlc_reader::{migration, server::Credential, update_pro_affiliations};
use std::error::Error;
use std::fs::File;
use std::io::{self, Write};
use std::path::PathBuf;
use std::{
    env,
    io::{BufReader, BufWriter},
};
use strum::{Display, EnumString};

struct JobConfig {
    tsv_dir: PathBuf,
    update_path: PathBuf,
    cred: Credential,
    should_trim: bool,
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
    step_count: usize,
) -> Result<(), Box<dyn Error>> {
    let steps = [
        RunStepConfig {
            step: RunStep::SaveDocs,
            f: &|_| migration::save_remote_mlc_docs(&config.cred, &config.tsv_dir),
        },
        RunStepConfig {
            step: RunStep::Schema,
            f: &|db| {
                migration::migrate_from_bwarm_dump(db, &config.tsv_dir)
                    .map_err(|e| e as Box<dyn Error>)
            },
        },
        RunStepConfig {
            step: RunStep::Trim,
            f: &|db| {
                if config.should_trim {
                    trim_db(db, true)?;
                } else {
                    log::info!("skipping trim step: creatind IDX instead");
                    setup_trim_indexes(db)?
                }
                Ok(())
            },
        },
        RunStepConfig {
            step: RunStep::IndexSearch,
            f: &|db| Ok(migration::create_relationship_indexes(db)?),
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
                // migration::add_party_stats(&tx)?;
                migration::enrich_work_totals(&tx)?;
                Ok(tx.commit()?)
            },
        },
        RunStepConfig {
            step: RunStep::UpdatePros,
            f: &|db| {
                let Ok(file) = std::fs::File::open(&config.update_path) else {
                    eprintln!("update path not found: skipping");
                    return Ok(());
                };

                let mut reader = BufReader::new(file);
                Ok(update_pro_affiliations(&mut reader, db)?)
            },
        },
    ];
    let mut num_steps_run = 0;
    for step in steps {
        if resume_step.as_ref().is_some_and(|r| *r > step.step) {
            continue;
        }

        // do nothing if step count is 0
        if step_count != 0 && num_steps_run >= step_count {
            break;
        }

        log::info!("Running Step: {}", step.step);
        (step.f)(conn)?;
        num_steps_run += 1;
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
    let tsv_dir: PathBuf = env::var("TSV_DIR").expect("missing TSV_DIR").into();
    let update_path: PathBuf = env::var("UPDATE_PATH").expect("missing UPDATE_PATH").into();

    let temp_db_url = env::var("TEMP_DB_URL").expect("missing TEMP_DB_URL");
    let prod_db_url = env::var("DB_URL").expect("missing DB_URL");
    let mut conn = rusqlite::Connection::open(&temp_db_url).expect("failed to connect to db");

    let args = Args::parse();
    match args.command {
        Command::Indexes {} => setup_trim_indexes(&conn).expect("failed to create indexes"),
        Command::Migrate { path } => {
            let conn = rusqlite::Connection::open(path).expect("failed to open db");
            migration::migrate_full_db(&conn).expect("failed to migrate");
        }
        Command::NightJob {
            resume_step,
            trim,
            step_count,
        } => {
            let config = JobConfig {
                tsv_dir,
                update_path,
                cred,
                should_trim: trim,
            };

            nightjob(&mut conn, config, resume_step, step_count).expect("NightJob failed");
            log::info!("promoting db to prod");
            promote_db(&temp_db_url, &prod_db_url).expect("failed to promote db");
            log::info!("nightjob complete");
        }
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
    Indexes {},
    Migrate {
        #[arg(short, long)]
        path: PathBuf,
    },
    NightJob {
        #[arg(short, long)]
        trim: bool,

        #[arg(short, long)]
        resume_step: Option<RunStep>,

        #[arg(short, long)]
        step_count: usize,
    },
}

fn promote_db(temp_path: &str, prod_path: &str) -> Result<(), Box<dyn Error>> {
    if temp_path == prod_path {
        return Ok(());
    }

    // ensure db path exists
    let prod_conn = rusqlite::Connection::open(prod_path)?;
    drop(prod_conn);

    let temp = File::open(temp_path)?;
    let prod = File::open(prod_path)?;
    let mut reader = BufReader::new(temp);
    let mut writer = BufWriter::new(prod);
    io::copy(&mut reader, &mut writer)?;
    writer.flush()?;
    Ok(())
}
