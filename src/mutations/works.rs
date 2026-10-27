use std::collections::HashMap;

use chrono::NaiveDate;
use musicmeta::{ipi::IpiNameNum, isrc::Isrc, iswc::Iswc, tis::society::TisSocietyCode};
use serde::{Deserialize, Serialize};

pub enum WorkSearch {
    Iswc(Iswc),
    Track { title: String, artist: String },
}

#[derive(Debug, Default, Serialize)]
pub struct PartyInfo {
    pub id: u64,
    pub full_name: String,
    pub ipi: Option<IpiNameNum>,
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
    pub iswc: Option<Iswc>,
    pub in_dispute: bool,
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
    pub isrc: Option<Isrc>,
    pub release_date: Option<NaiveDate>,
}
