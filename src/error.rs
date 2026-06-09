use axum::{
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};
use tera::Context;

pub enum AppError {
    Drova(drova_sdk::requester::Error),
    BadRequest(String),
    NotFound(String),
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
            AppError::NotFound(message) => (StatusCode::NOT_FOUND, message),
            AppError::Upstream(message) => (StatusCode::BAD_GATEWAY, message),
        };
        let mut context = Context::new();

        context.insert("status", &status.as_u16());
        context.insert("message", &message);

        let body = crate::templates::render("error.tera", &context)
            .unwrap_or_else(|_| format!("<h1>{}</h1><p>{}</p>", status.as_u16(), message));

        (status, Html(body)).into_response()
    }
}
