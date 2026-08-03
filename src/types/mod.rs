use std::{fmt::Display, str::FromStr};

use clap::ValueEnum;
use musicmeta::{ipi::IpiNameNum, tis::society::TisSocietyCode};
use serde::{Deserialize, Serialize};

pub mod territory;

#[derive(Debug, Deserialize, Serialize)]
pub enum WriterSearch {
    Ipi(IpiNameNum),
    Name(String),
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Party {
    pub id: u64,
    // pub full_name: String,
    pub first_name: Option<String>,
    pub last_name: String,
    pub ipi_name_num: Option<IpiNameNum>,
    pub pro: Option<TisSocietyCode>,
    pub role: Option<Role>,
}

#[derive(Debug, Deserialize, Serialize, Clone, ValueEnum)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    Publisher,
    Writer,
}
impl FromStr for Role {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let x = match s {
            "publisher" => Self::Publisher,
            "writer" => Self::Writer,
            _ => {
                return Err(());
            }
        };
        Ok(x)
    }
}
impl Display for Role {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match self {
            Role::Publisher => "publisher",
            Role::Writer => "writer",
        };
        write!(f, "{text}")
    }
}
