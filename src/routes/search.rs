use axum::{
    extract::{Query, State},
    response::Redirect,
};
use serde::Deserialize;

use crate::{error::AppError, state::AppState};

use super::url_encode;

#[derive(Deserialize)]
pub struct SearchParams {
    q: String,
}

pub async fn search(
    State(state): State<AppState>,
    Query(params): Query<SearchParams>,
) -> Result<Redirect, AppError> {
    let searx_url = state
        .config
        .third_party
        .searx_url
        .as_ref()
        .ok_or_else(|| AppError::BadRequest("search is disabled".to_string()))?;
    let target = format!("{}/search?q={}", searx_url, url_encode(&params.q));

    Ok(Redirect::temporary(&format!(
        "/get?url={}",
        url_encode(&target)
    )))
}
