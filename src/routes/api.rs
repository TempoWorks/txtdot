use axum::{
    extract::State,
    http::{HeaderMap, Uri},
    response::{Html, IntoResponse, Response},
    Json,
};
use base64::Engine;
use serde::Deserialize;

use crate::{drova, error::AppError, proxy, state::AppState};

use super::request_base;

#[derive(Deserialize)]
pub struct ApiRequest {
    url: Option<String>,
    input_type: Option<String>,
    output_type: Option<String>,
    text: Option<String>,
    bytes_base64: Option<String>,
    rewrite_links: Option<bool>,
    process_images: Option<bool>,
    proxy_documents: Option<bool>,
    proxy_images: Option<bool>,
    proxy_media: Option<bool>,
    proxy_files: Option<bool>,
    request_base: Option<String>,
}

pub async fn api_get(
    State(state): State<AppState>,
    headers: HeaderMap,
    uri: Uri,
    Json(request): Json<ApiRequest>,
) -> Result<Response, AppError> {
    let requester = drova::requester();
    let source_url = request.url.clone();
    let page = if let Some(url) = &request.url {
        requester.process(url).await?
    } else if let Some(text) = request.text {
        let input_type = request
            .input_type
            .as_deref()
            .ok_or_else(|| AppError::BadRequest("input_type is required".to_string()))?;
        requester.process_text(input_type, text)?
    } else if let Some(bytes_base64) = request.bytes_base64 {
        let input_type = request
            .input_type
            .as_deref()
            .ok_or_else(|| AppError::BadRequest("input_type is required".to_string()))?;
        let bytes = base64::engine::general_purpose::STANDARD
            .decode(bytes_base64)
            .map_err(|error| AppError::BadRequest(error.to_string()))?;
        requester.process_bytes(input_type, bytes)?
    } else {
        return Err(AppError::BadRequest(
            "url, text, or bytes_base64 is required".to_string(),
        ));
    };

    let output_type = request
        .output_type
        .as_deref()
        .unwrap_or(drova::DEFAULT_API_OUTPUT);
    let output_type = drova::resolve_output_type(output_type)?;

    if drova::is_html_output(&output_type) {
        let mut html = drova::render_html(page)?;
        if request.rewrite_links.unwrap_or(source_url.is_some()) {
            let remote_url = source_url.ok_or_else(|| {
                AppError::BadRequest("url is required when rewrite_links is true".to_string())
            })?;
            let request_base = request
                .request_base
                .unwrap_or_else(|| request_base(&headers, uri.scheme_str()));
            let process_images = state.config.proxy.process_images
                && request
                    .process_images
                    .unwrap_or(state.config.img_optimize_by_default);
            let rewrite_options = proxy::RewriteOptions {
                proxy_documents: request
                    .proxy_documents
                    .unwrap_or(state.config.proxy.documents)
                    && state.config.proxy.documents,
                proxy_images: request.proxy_images.unwrap_or(state.config.proxy.images)
                    && state.config.proxy.images,
                proxy_media: request.proxy_media.unwrap_or(state.config.proxy.media)
                    && state.config.proxy.media,
                proxy_files: request.proxy_files.unwrap_or(state.config.proxy.files)
                    && state.config.proxy.files,
                process_images,
            };
            html = proxy::rewrite_html_links(&html, &request_base, &remote_url, rewrite_options)?;
        }
        return Ok(Html(html).into_response());
    }

    drova::output_response(&output_type, page)
}
