use std::collections::HashMap;

use axum::{
    Json,
    extract::{Path, State},
};
use serde::Deserialize;
use strum::EnumString;

use crate::{
    api::{
        error::{ApiError, Envelope},
        state,
    },
    query::{self, types::WorkInfo},
};

pub async fn get_work(
    State(state): State<state::State>,
    Path(id): Path<String>,
) -> Result<Json<Envelope<Vec<WorkInfo>>>, ApiError> {
    let conn = rusqlite::Connection::open(&state.db_url)?;

    let works = query::works::get_works(&conn, &[&id], true)?;
    if works.is_empty() {
        return Err(ApiError::NotFound);
    }
    Ok(Envelope::ok(works))
}

#[derive(Deserialize)]
pub struct SearchWorksBody {
    pub identifiers: Vec<String>,
    pub method: WorkIdentifier,
}

#[derive(Deserialize, EnumString)]
pub enum WorkIdentifier {
    Isrc,
    Iswc,
}

pub async fn search_works(
    State(state): State<state::State>,
    Json(body): Json<SearchWorksBody>,
) -> Result<Json<Envelope<HashMap<String, Vec<WorkInfo>>>>, ApiError> {
    let conn = rusqlite::Connection::open(&state.db_url)?;

    let works = match body.method {
        WorkIdentifier::Isrc => {
            query::works::search_works_by_isrcs(&conn, body.identifiers.as_slice())?
        }
        WorkIdentifier::Iswc => {
            query::works::search_works_by_iswcs(&conn, body.identifiers.as_slice())?
        }
    };
    Ok(Envelope::ok(works))
}
