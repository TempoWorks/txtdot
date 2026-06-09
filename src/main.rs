use std::{
    net::{IpAddr, SocketAddr},
    sync::Arc,
};

use axum::{
    extract::{Query, State},
    http::{header, HeaderMap, HeaderValue, Uri},
    response::{Html, IntoResponse, Redirect, Response},
    routing::{get, post},
    Json, Router,
};
use base64::Engine;
use serde::Deserialize;
use serde_json::{json, Value};
use tera::Context;
use tower_http::services::ServeDir;

mod config;
mod drova;
mod error;
mod proxy;
mod templates;

use config::{Config, DESCRIPTION};
use error::AppError;

#[derive(Clone)]
struct AppState {
    config: Arc<Config>,
    routes: Arc<Vec<&'static str>>,
}

#[derive(Deserialize)]
struct GetParams {
    url: String,
    format: Option<String>,
}

#[derive(Deserialize)]
struct SearchParams {
    q: String,
}

#[derive(Deserialize)]
struct ApiRequest {
    url: Option<String>,
    input_type: Option<String>,
    output_type: Option<String>,
    text: Option<String>,
    bytes_base64: Option<String>,
    rewrite_links: Option<bool>,
    request_base: Option<String>,
}

#[tokio::main]
async fn main() {
    let config = Arc::new(Config::from_env());
    let addr = SocketAddr::from((
        config
            .host
            .parse::<IpAddr>()
            .unwrap_or_else(|_| [0, 0, 0, 0].into()),
        config.port,
    ));

    let app = Router::new()
        .route("/", get(index))
        .route("/get", get(get_page))
        .route("/search", get(search))
        .route("/configuration", get(configuration))
        .route("/configuration/json", get(configuration_json))
        .route("/configuration/badges/big", get(big_badge))
        .route("/api/get", post(api_get))
        .route("/proxy", get(proxy::proxy))
        .route("/proxy/img", get(proxy::proxy_img))
        .nest_service("/static", ServeDir::new(env!("TXTDOT_STATIC_DIR")))
        .with_state(AppState {
            config,
            routes: Arc::new(app_routes()),
        });

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("bind txtdot v2 server");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .expect("run txtdot v2 server");
}

async fn shutdown_signal() {
    let _ = tokio::signal::ctrl_c().await;
}

async fn index(State(state): State<AppState>) -> Result<Html<String>, AppError> {
    let mut context = Context::new();
    context.insert("description", DESCRIPTION);
    context.insert("version", env!("CARGO_PKG_VERSION"));
    context.insert("search", &state.config.third_party.searx_url.is_some());
    context.insert("search_by_default", &state.config.search_by_default);
    context.insert("focus", &true);
    context.insert("engines", &["drova"]);

    Ok(Html(templates::render("index.tera", &context)?))
}

async fn get_page(
    State(state): State<AppState>,
    headers: HeaderMap,
    uri: Uri,
    Query(params): Query<GetParams>,
) -> Result<Response, AppError> {
    let page = drova::requester().process(&params.url).await?;
    let format = params.format.as_deref().unwrap_or("html");

    if matches!(format, "dalet" | "daletpack" | "application/daletpack") {
        return daletpack_response(drova::render_daletpack(page)?);
    }

    let content = proxy::rewrite_html_links(
        &drova::render_html(page)?,
        &request_base(&headers, uri.scheme_str()),
        &params.url,
        state.config.proxy.img_compress,
    )?;
    let mut context = Context::new();
    context.insert(
        "parsed",
        &json!({
            "lang": "en",
            "title": "txt. parsed page",
            "content": content,
        }),
    );
    context.insert("remote_url", &params.url);
    context.insert("search", &false);

    Ok(Html(templates::render("get.tera", &context)?).into_response())
}

async fn search(
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

    Ok(Redirect::temporary(&format!("/get?url={}", url_encode(&target))))
}

async fn configuration(State(state): State<AppState>) -> Result<Html<String>, AppError> {
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
    context.insert(
        "engines",
        &[json!({
            "name": "drova",
            "description": "Drova protocol/input/output plugin registry",
        })],
    );
    context.insert("middlewares", &Vec::<Value>::new());
    context.insert("routes", state.routes.as_ref());

    Ok(Html(templates::render("configuration.tera", &context)?))
}

async fn big_badge(State(state): State<AppState>, headers: HeaderMap) -> Result<Response, AppError> {
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

async fn configuration_json(State(state): State<AppState>) -> Json<Value> {
    Json(json!({
        "version": env!("CARGO_PKG_VERSION"),
        "config": public_config(state.config.as_ref()),
        "protocols": protocols(),
        "inputs": inputs(),
        "outputs": outputs(),
        "routes": state.routes.as_ref(),
    }))
}

async fn api_get(
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

    let output_type = request.output_type.as_deref().unwrap_or("application/daletpack");
    match output_type {
        "html" | "text/html" => {
            let mut html = drova::render_html(page)?;
            if request.rewrite_links.unwrap_or(source_url.is_some()) {
                let remote_url = source_url.ok_or_else(|| {
                    AppError::BadRequest("url is required when rewrite_links is true".to_string())
                })?;
                let request_base = request
                    .request_base
                    .unwrap_or_else(|| request_base(&headers, uri.scheme_str()));
                html = proxy::rewrite_html_links(
                    &html,
                    &request_base,
                    &remote_url,
                    state.config.proxy.img_compress,
                )?;
            }
            Ok(Html(html).into_response())
        }
        "dalet" | "daletpack" | "application/daletpack" => {
            daletpack_response(drova::render_daletpack(page)?)
        }
        other => Err(AppError::BadRequest(format!(
            "unsupported output format: {other}"
        ))),
    }
}

fn daletpack_response(bytes: Vec<u8>) -> Result<Response, AppError> {
    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/daletpack"),
    );
    Ok((headers, bytes).into_response())
}

fn request_base(headers: &HeaderMap, scheme: Option<&str>) -> String {
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

fn public_config(config: &Config) -> Value {
    let mut value = serde_json::to_value(config).unwrap_or_else(|_| json!({}));
    if let Some(object) = value.as_object_mut() {
        object.remove("host");
        object.remove("port");
    }
    value
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
    context.insert("engines_count", &1);
    context.insert("middlewares_count", &0);

    templates::render("big-badge.tera", &context)
}

fn protocols() -> Vec<&'static str> {
    vec!["http", "https", "gemini", "gopher"]
}

fn inputs() -> Vec<&'static str> {
    vec![
        "application/daletpack",
        "text/gemini",
        "text/x-gophermap",
        "text/markdown",
        "text/x-markdown",
        "text/html",
        "text/plain",
        "text/*",
    ]
}

fn outputs() -> Vec<&'static str> {
    vec!["application/daletpack", "text/html"]
}

fn url_encode(value: &str) -> String {
    url::form_urlencoded::byte_serialize(value.as_bytes()).collect()
}

fn app_routes() -> Vec<&'static str> {
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
