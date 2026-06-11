use clap::{Parser, ValueEnum};
use cwr::models::society::SocietyCode;
use dotenv::dotenv;
use libsql::{Builder, Database};
use serde::Deserialize;
use std::{
    env,
    io::{BufRead, BufReader},
    path::PathBuf,
};

use crate::{
    additional::migrate_add_ons, save::migrate_from_bwarm_dump, update::update_publisher_writers,
};

#[derive(Debug, Deserialize, Clone, ValueEnum)]
pub enum EnrichMode {
    Writer,
    Publisher,
    Role,
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(clap::Subcommand, Debug)]
pub enum Command {
    Migrate {},
    Modify {},

    Enrich {
        #[arg(short, long)]
        role: EnrichMode,
    },
    Update {
        #[arg(short, long)]
        path: PathBuf,
    },
}
