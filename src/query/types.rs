use std::{fmt::Display, str::FromStr};

use chrono::NaiveDate;
use musicmeta::{ipi::IpiNameNum, isrc::Isrc, iswc::Iswc, tis::society::TisSocietyCode};
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct PartyMeta {
    pub info: Party,
    pub isrcs: Vec<String>,
    pub num_works: usize,
    pub total_shares: f32,
    pub avg_share: f32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Party {
    pub id: u64,
    pub first_name: Option<String>,
    pub last_name: String,
    pub ipi_name_num: Option<u64>, // no need for validation here (slower)
    pub pro: Option<TisSocietyCode>, // validation for beautify output
    pub role: Option<Role>,        // same here ^
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum PartyInput {
    Name(String),
    Ipi(IpiNameNum),
}

impl From<String> for PartyInput {
    fn from(value: String) -> Self {
        match value
            .parse::<u64>()
            .ok()
            .and_then(|x| IpiNameNum::new(x).ok())
        {
            Some(ipi) => PartyInput::Ipi(ipi),
            None => PartyInput::Name(value),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, clap::ValueEnum)]
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

pub enum WorkSearch {
    Iswc(Iswc),
    Track { title: String, artist: String },
}

#[derive(Debug, Default, Serialize)]
pub struct PartyInfo {
    pub id: u64,
    pub full_name: String,
    pub ipi: Option<u64>,
    pub pro: Option<TisSocietyCode>,
    pub shares: Vec<Share>,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Share {
    pub id: String,
    pub party_id: u64,
    pub role: String,
    // pub share_type: Option<String>,
    pub rights_type: Option<String>,
    pub share: f64,
    pub preceding_id: Option<String>,
    // pub territory: String,
}

#[derive(Debug, Default, Serialize)]
pub struct WorkInfo {
    pub id: String,
    pub title: String,
    pub duration_ms: Option<f64>,
    pub iswc: Option<String>,
    pub in_dispute: bool,
    pub total_shares: f64,
    pub releases: Vec<Release>,
    pub parties: Vec<PartyInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Release {
    pub id: i64,
    pub title: String,
    pub artist_name: String,
    pub label_name: String,
    pub distro_name: String,
    pub isrc: Option<String>,
    pub release_date: Option<NaiveDate>,
}

#[derive(Debug)]
pub struct WorkSearchParams {
    pub iswc: Option<Iswc>,
    pub isrc: Option<Isrc>,
    // to avoid dynamic sql this is not null
    // TODO: make this optional and use query builder
    pub title: String,
    pub artist: Option<String>,
    // to avoid dynamic sql this is not null
    // TODO: make this optional and use query builder
    pub party_ipi: IpiNameNum,
    pub limit: usize,
    pub offset: usize,
}
