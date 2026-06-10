use axum::{
    http::{header, HeaderMap},
    routing::{get, post},
    Router,
};
use serde_json::{json, Value};

use crate::{config::Config, state::AppState};

pub mod api;
pub mod configuration;
pub mod get_page;
pub mod index;
pub mod not_found;
pub mod search;

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/", get(index::index))
        .route("/get", get(get_page::get_page))
        .route("/search", get(search::search))
        .route("/configuration", get(configuration::configuration))
        .route(
            "/configuration/json",
            get(configuration::configuration_json),
        )
        .route("/configuration/badges/big", get(configuration::big_badge))
        .route("/api/get", post(api::api_get))
        .route("/proxy", get(crate::proxy::proxy))
        .route("/proxy/img", get(crate::proxy::proxy_img))
        .fallback(not_found::not_found)
        .with_state(state)
}

pub fn app_routes() -> Vec<&'static str> {
    vec![
        "/",
        "/get",
        "/search",
        "/configuration",
        "/configuration/json",
        "/configuration/badges/big",
        "/api/get",
        "/proxy",
        "/proxy/img",
    ]
}

pub fn public_config(config: &Config) -> Value {
    let mut value = serde_json::to_value(config).unwrap_or_else(|_| json!({}));
    if let Some(object) = value.as_object_mut() {
        object.remove("host");
        object.remove("port");
    }
    value
}

pub fn request_base(headers: &HeaderMap, scheme: Option<&str>) -> String {
    let scheme = headers
        .get("x-forwarded-proto")
        .and_then(|value| value.to_str().ok())
        .or(scheme)
        .unwrap_or("http");
    let host = headers
        .get(header::HOST)
        .and_then(|value| value.to_str().ok())
        .unwrap_or("127.0.0.1:8080");

    format!("{scheme}://{host}")
}

pub fn url_encode(value: &str) -> String {
    url::form_urlencoded::byte_serialize(value.as_bytes()).collect()
}
