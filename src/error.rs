use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

pub enum AppError {
    Drova(drova_sdk::requester::Error),
    BadRequest(String),
    Upstream(String),
}

impl From<drova_sdk::requester::Error> for AppError {
    fn from(error: drova_sdk::requester::Error) -> Self {
        Self::Drova(error)
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::Drova(error) => (StatusCode::BAD_GATEWAY, format!("{error:?}")),
            AppError::BadRequest(message) => (StatusCode::BAD_REQUEST, message),
            AppError::Upstream(message) => (StatusCode::BAD_GATEWAY, message),
        };

        (
            status,
            Json(json!({
                "data": null,
                "error": message,
            })),
        )
            .into_response()
    }
}
