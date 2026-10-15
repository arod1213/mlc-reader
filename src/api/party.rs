use axum::{
    Json,
    extract::{Path, Query, State},
};
use serde::{Deserialize, Serialize};

use crate::{
    api::{
        error::{ApiError, Envelope},
        state,
    },
    query::{
        self,
        types::{Party, PartyInput, WorkInfo},
    },
};

pub async fn get_party(
    State(state): State<state::State>,
    Path(id): Path<u64>,
) -> Result<Json<Envelope<Party>>, ApiError> {
    let conn = rusqlite::Connection::open(&state.db_url)?;

    let party = query::party::get_party(&conn, id)?;
    Ok(Envelope::ok(party))
}

#[derive(Serialize)]
pub struct PartyCollaborators {
    pub writers: Vec<Party>,
    pub publishers: Vec<Party>,
}

pub async fn get_party_collaborators(
    State(state): State<state::State>,
    Path(id): Path<u64>,
) -> Result<Json<Envelope<PartyCollaborators>>, ApiError> {
    let conn = rusqlite::Connection::open(&state.db_url)?;

    let publishers = query::party::get_party_collaborators(&conn, id, true)?;
    let writers = query::party::get_party_collaborators(&conn, id, false)?;
    let collab = PartyCollaborators {
        writers,
        publishers,
    };
    Ok(Envelope::ok(collab))
}

#[derive(Deserialize)]
pub struct PartyWorksQuery {
    #[serde(default)]
    pub with_id: Option<u64>,
}

pub async fn get_party_works(
    State(state): State<state::State>,
    Query(query): Query<PartyWorksQuery>,
    Path(id): Path<u64>,
) -> Result<Json<Envelope<Vec<WorkInfo>>>, ApiError> {
    let conn = rusqlite::Connection::open(&state.db_url)?;

    let works = query::works::get_party_works(&conn, id, query.with_id, true)?;
    Ok(Envelope::ok(works))
}

pub async fn search_parties(
    State(state): State<state::State>,
    Path(key): Path<String>,
) -> Result<Json<Envelope<Vec<Party>>>, ApiError> {
    let conn = rusqlite::Connection::open(&state.db_url)?;
    let input = PartyInput::from(key);

    let parties = query::party::search_parties(&conn, input)?;
    Ok(Envelope::ok(parties))
}
