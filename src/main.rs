use std::{
    net::{IpAddr, SocketAddr},
    sync::Arc,
};

use axum::{
    extract::{Query, State},
    http::{header, HeaderMap, HeaderValue, Uri},
    response::{Html, IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use base64::Engine;
use serde::{Deserialize, Serialize};
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
struct ConvertRequest {
    input_type: String,
    output_type: Option<String>,
    text: Option<String>,
    bytes_base64: Option<String>,
}

#[derive(Serialize)]
struct ApiResponse<T: Serialize> {
    data: T,
    error: Option<String>,
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
        .route("/configuration", get(configuration))
        .route("/configuration/json", get(configuration_json))
        .route("/api/parse", get(api_parse))
        .route("/api/raw-html", get(api_raw_html))
        .route("/api/convert", post(api_convert))
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

async fn configuration(State(state): State<AppState>) -> Result<Html<String>, AppError> {
    let env_pretty = serde_json::to_string_pretty(state.config.as_ref()).unwrap_or_default();
    let mut context = Context::new();
    context.insert("description", DESCRIPTION);
    context.insert("version", env!("CARGO_PKG_VERSION"));
    context.insert("env_pretty", &env_pretty);
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

async fn configuration_json(State(state): State<AppState>) -> Json<Value> {
    Json(json!({
        "version": env!("CARGO_PKG_VERSION"),
        "config": state.config.as_ref(),
        "protocols": ["http", "https", "gemini", "gopher"],
        "inputs": ["application/daletpack", "text/gemini", "text/x-gophermap", "text/markdown", "text/x-markdown", "text/html", "text/plain", "text/*"],
        "outputs": ["application/daletpack", "text/html"],
        "routes": state.routes.as_ref(),
    }))
}

async fn api_parse(Query(params): Query<GetParams>) -> Result<Response, AppError> {
    let page = drova::requester().process(&params.url).await?;
    let format = params.format.as_deref().unwrap_or("daletpack");

    if matches!(format, "dalet" | "daletpack" | "application/daletpack") {
        return daletpack_response(drova::render_daletpack(page)?);
    }

    let data = drova::output_json(format, page)?;
    Ok(Json(ApiResponse { data, error: None }).into_response())
}

async fn api_raw_html(
    State(state): State<AppState>,
    headers: HeaderMap,
    uri: Uri,
    Query(params): Query<GetParams>,
) -> Result<Html<String>, AppError> {
    let page = drova::requester().process(&params.url).await?;
    Ok(Html(proxy::rewrite_html_links(
        &drova::render_html(page)?,
        &request_base(&headers, uri.scheme_str()),
        &params.url,
        state.config.proxy.img_compress,
    )?))
}

async fn api_convert(Json(request): Json<ConvertRequest>) -> Result<Response, AppError> {
    let requester = drova::requester();
    let page = if let Some(text) = request.text {
        requester.process_text(&request.input_type, text)?
    } else if let Some(bytes_base64) = request.bytes_base64 {
        let bytes = base64::engine::general_purpose::STANDARD
            .decode(bytes_base64)
            .map_err(|error| AppError::BadRequest(error.to_string()))?;
        requester.process_bytes(&request.input_type, bytes)?
    } else {
        return Err(AppError::BadRequest(
            "text or bytes_base64 is required".to_string(),
        ));
    };

    let output_type = request.output_type.as_deref().unwrap_or("application/daletpack");
    match output_type {
        "html" | "text/html" => Ok(Html(drova::render_html(page)?).into_response()),
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

fn app_routes() -> Vec<&'static str> {
    vec![
        "/",
        "/get",
        "/configuration",
        "/configuration/json",
        "/api/parse",
        "/api/raw-html",
        "/api/convert",
        "/proxy",
        "/proxy/img",
    ]
}
