use axum::{
    extract::{Query, State},
    http::{HeaderMap, Uri},
    response::{Html, IntoResponse, Response},
};
use serde::Deserialize;
use serde_json::json;
use tera::Context;

use crate::{drova, error::AppError, proxy, state::AppState, templates};

use super::request_base;

#[derive(Deserialize)]
pub struct GetParams {
    url: String,
    format: Option<String>,
}

pub async fn get_page(
    State(state): State<AppState>,
    headers: HeaderMap,
    uri: Uri,
    Query(params): Query<GetParams>,
) -> Result<Response, AppError> {
    let page = drova::requester().process(&params.url).await?;
    let format = params.format.as_deref().unwrap_or("html");

    if matches!(format, "dalet" | "daletpack" | "application/daletpack") {
        return drova::daletpack_response(drova::render_daletpack(page));
    }

    let content = proxy::rewrite_html_links(
        &drova::render_html(page.clone())?,
        &request_base(&headers, uri.scheme_str()),
        &params.url,
        state.config.proxy.img_compress,
    )?;
    let mut context = Context::new();
    context.insert(
        "parsed",
        &json!({
            "lang": "en",
            "title": page.title,
            "content": content,
        }),
    );
    context.insert("remote_url", &params.url);
    context.insert("search", &false);

    Ok(Html(templates::render("get.tera", &context)?).into_response())
}
