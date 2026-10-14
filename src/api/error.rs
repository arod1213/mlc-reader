use axum::{Json, http::StatusCode, response::IntoResponse};
use serde::Serialize;

#[derive(Serialize)]
pub struct Envelope<T: Serialize> {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl<T: Serialize> Envelope<T> {
    pub fn ok(data: T) -> Json<Self> {
        Json(Self {
            success: true,
            message: None,
            data: Some(data),
            error: None,
        })
    }
}

#[derive(Debug)]
pub enum ApiError {
    Db(rusqlite::Error),
    NotFound,
}

impl From<rusqlite::Error> for ApiError {
    fn from(e: rusqlite::Error) -> Self {
        match e {
            rusqlite::Error::QueryReturnedNoRows => ApiError::NotFound,
            e => ApiError::Db(e),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let (status, message, error) = match self {
            ApiError::Db(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "database error",
                Some(e.to_string()),
            ),
            ApiError::NotFound => (StatusCode::NOT_FOUND, "not found", None),
        };
        if status.is_server_error() {
            log::error!("{status} {message}: {error:?}");
        } else {
            log::warn!("{status} {message}: {error:?}");
        }

        let body = Envelope::<()> {
            success: false,
            message: Some(message.to_string()),
            data: None,
            error,
        };
        (status, Json(body)).into_response()
    }
}
