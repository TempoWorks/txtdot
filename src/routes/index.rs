use axum::{extract::State, response::Html};
use tera::Context;

use crate::{config::DESCRIPTION, error::AppError, state::AppState, templates};

pub async fn index(State(state): State<AppState>) -> Result<Html<String>, AppError> {
    let mut context = Context::new();
    context.insert("description", DESCRIPTION);
    context.insert("version", env!("CARGO_PKG_VERSION"));
    context.insert("search", &state.config.third_party.searx_url.is_some());
    context.insert("search_by_default", &state.config.search_by_default);
    context.insert("focus", &true);

    Ok(Html(templates::render("index.tera", &context)?))
}
