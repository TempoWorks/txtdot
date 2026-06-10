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
    highlight_code: Option<bool>,
    optimize_images: Option<bool>,
    skip_image_optimization: Option<bool>,
    direct_documents: Option<bool>,
    direct_images: Option<bool>,
    direct_media: Option<bool>,
    direct_files: Option<bool>,
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

    let mut redirect = format!("/get?url={}", url_encode(&target));
    append_bool_param(&mut redirect, "highlight_code", params.highlight_code);
    append_bool_param(&mut redirect, "optimize_images", params.optimize_images);
    append_bool_param(
        &mut redirect,
        "skip_image_optimization",
        params.skip_image_optimization,
    );
    append_bool_param(&mut redirect, "direct_documents", params.direct_documents);
    append_bool_param(&mut redirect, "direct_images", params.direct_images);
    append_bool_param(&mut redirect, "direct_media", params.direct_media);
    append_bool_param(&mut redirect, "direct_files", params.direct_files);

    Ok(Redirect::temporary(&redirect))
}

fn append_bool_param(target: &mut String, name: &str, value: Option<bool>) {
    if let Some(value) = value {
        target.push('&');
        target.push_str(name);
        target.push('=');
        target.push_str(if value { "true" } else { "false" });
    }
}
