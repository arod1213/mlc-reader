use libsql::{Connection, Database, params};
use musicmeta::{ipi::IpiNameNum, tis::society::TisSocietyCode};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub enum WriterSearch {
    Ipi(IpiNameNum),
    Name(String),
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Party {
    pub full_name: String,
    pub ipi_name_num: Option<IpiNameNum>,
    pub pro: Option<TisSocietyCode>,
}
