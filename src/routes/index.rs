use axum::{extract::State, response::Html};
use tera::Context;

use crate::{config::DESCRIPTION, error::AppError, state::AppState, templates};

pub async fn index(State(state): State<AppState>) -> Result<Html<String>, AppError> {
    let mut context = Context::new();
    context.insert("description", DESCRIPTION);
    context.insert("version", env!("CARGO_PKG_VERSION"));
    context.insert("search", &state.config.third_party.searx_url.is_some());
    context.insert("search_by_default", &state.config.search_by_default);
    context.insert(
        "image_processing_available",
        &state.config.proxy.process_images,
    );
    context.insert(
        "image_optimization_default",
        &state.config.img_optimize_by_default,
    );
    context.insert(
        "process_images",
        &(state.config.proxy.process_images && state.config.img_optimize_by_default),
    );
    context.insert("proxy_documents", &state.config.proxy.documents);
    context.insert("proxy_images", &state.config.proxy.images);
    context.insert("proxy_media", &state.config.proxy.media);
    context.insert("proxy_files", &state.config.proxy.files);
    context.insert("server_proxy_documents", &state.config.proxy.documents);
    context.insert("server_proxy_images", &state.config.proxy.images);
    context.insert("server_proxy_media", &state.config.proxy.media);
    context.insert("server_proxy_files", &state.config.proxy.files);
    context.insert("focus", &true);

    Ok(Html(templates::render("index.tera", &context)?))
}
