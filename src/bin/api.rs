use std::env;

use axum::{
    Json, Router,
    extract::State,
    http::{Method, StatusCode},
    response::IntoResponse,
    routing::get,
};
use dotenv::dotenv;
use mlc_reader::api::{
    self,
    party::{get_party, get_party_collaborators, get_party_works, search_parties},
    work::{get_work, search_works_by_codes},
};
use tower_http::cors::{Any, CorsLayer};

/// Simple health check to ensure API is live
async fn health(State(state): State<api::state::State>) -> impl IntoResponse {
    match rusqlite::Connection::open(state.db_url) {
        Ok(_) => (StatusCode::OK, Json("ok")).into_response(),
        Err(e) => (StatusCode::SERVICE_UNAVAILABLE, Json(e.to_string())).into_response(),
    }
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    env_logger::init();

    let port = env::var("PORT").unwrap_or("8080".into());
    let port: u32 = port.parse().expect("PORT must be a valid number");

    let db_url = env::var("DB_URL").expect("missing DB_URL");
    let state = api::state::State { db_url };

    // let origins = ["http://localhost:5173".parse().unwrap()];
    let cors = CorsLayer::new()
        .allow_methods([
            Method::GET,
            // Method::POST,
            // Method::PUT
        ])
        .allow_headers(Any)
        .allow_origin(Any)
        .expose_headers([axum::http::header::CONTENT_DISPOSITION]);

    let api = Router::new()
        // find works by ISRCs or ISWCs
        .route("/works", get(search_works_by_codes))
        // get one work (with more info)
        .route("/work/{id}", get(get_work))
        // find parties by Name or IPI
        .route("/parties/{key}", get(search_parties))
        // get party details
        .route("/party/{id}", get(get_party))
        // get party details
        .route("/party/{id}/collaborators", get(get_party_collaborators))
        // get party works
        .route("/party/{id}/works", get(get_party_works));

    let app = Router::new()
        .route("/health", get(health))
        .nest("/api", api)
        .with_state(state)
        .layer(cors);

    let listen_on = format!("0.0.0.0:{}", port);
    let listener = tokio::net::TcpListener::bind(listen_on).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
