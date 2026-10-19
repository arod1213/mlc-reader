use std::collections::HashMap;

use axum::{
    Json,
    extract::{Path, Query, State},
};
use rusqlite::OpenFlags;
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
    let conn =
        rusqlite::Connection::open_with_flags(&state.db_url, OpenFlags::SQLITE_OPEN_READ_ONLY)?;

    let works = query::works::get_works(&conn, &[&id], 1_000, true)?;
    if works.is_empty() {
        return Err(ApiError::NotFound);
    }
    Ok(Envelope::ok(works))
}

#[derive(Deserialize)]
pub struct SearchWorksQuery {
    pub title: String,
    pub artist: Option<String>,
}

pub async fn search_works_by_name(
    State(state): State<state::State>,
    Query(query): Query<SearchWorksQuery>,
) -> Result<Json<Envelope<Vec<WorkInfo>>>, ApiError> {
    let conn =
        rusqlite::Connection::open_with_flags(&state.db_url, OpenFlags::SQLITE_OPEN_READ_ONLY)?;

    let works = query::works::search_works_by_title(&conn, query.artist.as_deref(), &query.title)?;
    Ok(Envelope::ok(works))
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkIdentifier {
    Isrc,
    Iswc,
}

#[derive(Deserialize)]
pub struct SearchWorksBody {
    pub method: WorkIdentifier,
    pub codes: Vec<String>,
}

pub async fn search_works_by_codes(
    State(state): State<state::State>,
    Json(body): Json<SearchWorksBody>,
) -> Result<Json<Envelope<HashMap<String, Vec<WorkInfo>>>>, ApiError> {
    let conn =
        rusqlite::Connection::open_with_flags(&state.db_url, OpenFlags::SQLITE_OPEN_READ_ONLY)?;

    let works = match body.method {
        WorkIdentifier::Isrc => query::works::search_works_by_isrcs(&conn, body.codes.as_slice())?,
        WorkIdentifier::Iswc => query::works::search_works_by_iswcs(&conn, body.codes.as_slice())?,
    };
    Ok(Envelope::ok(works))
}
