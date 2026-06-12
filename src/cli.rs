use clap::{Parser, ValueEnum};
use serde::Deserialize;
use std::path::PathBuf;

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
    Discover {
        #[arg(short, long)]
        method: DiscoverMode,
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
