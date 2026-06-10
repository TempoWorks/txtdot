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
    highlight_code: Option<bool>,
    optimize_images: Option<bool>,
    skip_image_optimization: Option<bool>,
    direct_documents: Option<bool>,
    direct_images: Option<bool>,
    direct_media: Option<bool>,
    direct_files: Option<bool>,
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

    let proxy_documents = state.config.proxy.documents && !params.direct_documents.unwrap_or(false);
    let proxy_images = state.config.proxy.images && !params.direct_images.unwrap_or(false);
    let proxy_media = state.config.proxy.media && !params.direct_media.unwrap_or(false);
    let proxy_files = state.config.proxy.files && !params.direct_files.unwrap_or(false);
    let requested_image_optimization = if state.config.img_optimize_by_default {
        !params.skip_image_optimization.unwrap_or(false)
    } else {
        params.optimize_images.unwrap_or(false)
    };
    let process_images = state.config.proxy.process_images && requested_image_optimization;
    let rewrite_options = proxy::RewriteOptions {
        proxy_documents,
        proxy_images,
        proxy_media,
        proxy_files,
        process_images,
    };
    let content = proxy::rewrite_html_links(
        &drova::render_html(page.clone())?,
        &request_base(&headers, uri.scheme_str()),
        &params.url,
        rewrite_options,
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
    context.insert("highlight_code", &params.highlight_code.unwrap_or(false));
    context.insert("search", &state.config.third_party.searx_url.is_some());
    context.insert(
        "image_processing_available",
        &state.config.proxy.process_images,
    );
    context.insert(
        "image_optimization_default",
        &state.config.img_optimize_by_default,
    );
    context.insert("process_images", &process_images);
    context.insert("proxy_documents", &proxy_documents);
    context.insert("proxy_images", &proxy_images);
    context.insert("proxy_media", &proxy_media);
    context.insert("proxy_files", &proxy_files);
    context.insert("server_proxy_documents", &state.config.proxy.documents);
    context.insert("server_proxy_images", &state.config.proxy.images);
    context.insert("server_proxy_media", &state.config.proxy.media);
    context.insert("server_proxy_files", &state.config.proxy.files);

    Ok(Html(templates::render("get.tera", &context)?).into_response())
}
