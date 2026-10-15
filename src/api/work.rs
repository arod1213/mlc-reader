use std::collections::HashMap;

use axum::{
    Json,
    extract::{Path, Query, State},
};
use serde::Deserialize;

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
#[serde(rename_all = "snake_case")]
pub enum WorkIdentifier {
    Isrc,
    Iswc,
}

#[derive(Deserialize)]
pub struct SearchWorksQuery {
    pub method: WorkIdentifier,
    pub codes: Vec<String>,
}

pub async fn search_works_by_codes(
    State(state): State<state::State>,
    Query(query): Query<SearchWorksQuery>,
) -> Result<Json<Envelope<HashMap<String, Vec<WorkInfo>>>>, ApiError> {
    let conn = rusqlite::Connection::open(&state.db_url)?;

    let works = match query.method {
        WorkIdentifier::Isrc => query::works::search_works_by_isrcs(&conn, query.codes.as_slice())?,
        WorkIdentifier::Iswc => query::works::search_works_by_iswcs(&conn, query.codes.as_slice())?,
    };
    Ok(Envelope::ok(works))
}
