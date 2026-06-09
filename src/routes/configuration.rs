use axum::{
    extract::State,
    http::{header, HeaderMap, HeaderValue},
    response::{Html, IntoResponse, Response},
    Json,
};
use serde_json::{json, Value};
use tera::Context;

use crate::{config::DESCRIPTION, error::AppError, state::AppState, templates};

use super::{inputs, outputs, protocols, public_config, request_base};

pub async fn configuration(State(state): State<AppState>) -> Result<Html<String>, AppError> {
    let env_pretty = serde_json::to_string_pretty(&public_config(state.config.as_ref()))
        .unwrap_or_default()
        .replace(['{', '}', '[', ']', '"', ','], "");
    let mut context = Context::new();
    let badge = render_big_badge(&state, "txtdot")?;

    context.insert("description", DESCRIPTION);
    context.insert("version", env!("CARGO_PKG_VERSION"));
    context.insert("env_pretty", &env_pretty);
    context.insert("badge", &badge);
    context.insert("protocols", &protocols());
    context.insert("inputs", &inputs());
    context.insert("outputs", &outputs());
    context.insert("routes", state.routes.as_ref());

    Ok(Html(templates::render("configuration.tera", &context)?))
}

pub async fn big_badge(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Response, AppError> {
    let mut response = Html(render_big_badge(
        &state,
        &request_base(&headers, Some("http")),
    )?)
    .into_response();
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("image/svg+xml"),
    );
    Ok(response)
}

pub async fn configuration_json(State(state): State<AppState>) -> Json<Value> {
    Json(json!({
        "version": env!("CARGO_PKG_VERSION"),
        "config": public_config(state.config.as_ref()),
        "protocols": protocols(),
        "inputs": inputs(),
        "outputs": outputs(),
        "routes": state.routes.as_ref(),
    }))
}

fn render_big_badge(state: &AppState, base_url: &str) -> Result<String, AppError> {
    let mut context = Context::new();
    context.insert("version", env!("CARGO_PKG_VERSION"));
    context.insert("base_url", base_url);
    context.insert(
        "search",
        if state.config.third_party.searx_url.is_some() {
            "enabled"
        } else {
            "disabled"
        },
    );
    context.insert("inputs_count", &inputs().len());
    context.insert("outputs_count", &outputs().len());

    templates::render("big-badge.tera", &context)
}
