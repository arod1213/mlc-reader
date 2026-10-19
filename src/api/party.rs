use axum::{
    Json,
    extract::{Path, Query, State},
};
use rusqlite::OpenFlags;
use serde::{Deserialize, Serialize};

use crate::{
    api::{
        error::{ApiError, Envelope},
        state,
    },
    query::{
        self,
        types::{Party, PartyInput, PartyMeta, WorkInfo},
    },
};

pub async fn get_party(
    State(state): State<state::State>,
    Path(id): Path<u64>,
) -> Result<Json<Envelope<Party>>, ApiError> {
    let conn =
        rusqlite::Connection::open_with_flags(&state.db_url, OpenFlags::SQLITE_OPEN_READ_ONLY)?;

    let party = query::party::get_party(&conn, id)?;
    Ok(Envelope::ok(party))
}

#[derive(Serialize)]
pub struct RepresentedParties {
    pub writers: Vec<Party>,
    pub publishers: Vec<Party>,
}

#[derive(Serialize)]
pub struct PartyCollaborators {
    pub represented: RepresentedParties,
    pub collaborators: Vec<Party>,
}

pub async fn get_party_collaborators(
    State(state): State<state::State>,
    Path(id): Path<u64>,
) -> Result<Json<Envelope<PartyCollaborators>>, ApiError> {
    let conn =
        rusqlite::Connection::open_with_flags(&state.db_url, OpenFlags::SQLITE_OPEN_READ_ONLY)?;

    let publishers = query::party::get_party_chain_of_title(&conn, id, true)?;
    let writers = query::party::get_party_chain_of_title(&conn, id, false)?;
    let collaborators = query::party::get_party_collaborators(&conn, id)?;
    let res = PartyCollaborators {
        represented: RepresentedParties {
            writers,
            publishers,
        },
        collaborators,
    };
    Ok(Envelope::ok(res))
}

pub async fn get_party_works(
    State(state): State<state::State>,
    Query(query): Query<query::works::PartyWorksQuery>,
    Path(id): Path<u64>,
) -> Result<Json<Envelope<Vec<WorkInfo>>>, ApiError> {
    let conn =
        rusqlite::Connection::open_with_flags(&state.db_url, OpenFlags::SQLITE_OPEN_READ_ONLY)?;

    let works = query::works::get_party_works(&conn, id, query)?;
    Ok(Envelope::ok(works))
}

pub async fn search_parties(
    State(state): State<state::State>,
    Path(key): Path<String>,
) -> Result<Json<Envelope<Vec<Party>>>, ApiError> {
    let conn =
        rusqlite::Connection::open_with_flags(&state.db_url, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
    let input = PartyInput::from(key);

    let parties = query::party::search_parties(&conn, input)?;
    Ok(Envelope::ok(parties))
}

#[derive(Deserialize)]
pub struct PartyRankBody {
    pub isrcs: Vec<String>,
}

pub async fn get_party_ranks(
    State(state): State<state::State>,
    Json(body): Json<PartyRankBody>,
) -> Result<Json<Envelope<Vec<PartyMeta>>>, ApiError> {
    let conn =
        rusqlite::Connection::open_with_flags(&state.db_url, OpenFlags::SQLITE_OPEN_READ_ONLY)?;

    let res = query::party::parties_by_market_share(&conn, &body.isrcs)?;
    Ok(Envelope::ok(res))
}
