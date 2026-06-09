use std::{
    env,
    net::{IpAddr, SocketAddr},
};

use axum::{
    extract::{Query, State},
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::{Html, IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use base64::Engine;
use dalet::types::Page;
use drova_plugins::requester_plugins;
use drova_sdk::requester::{OutputData, Requester, RequesterBuilder};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tower_http::services::ServeDir;

#[derive(Clone)]
struct AppState {
    config: Config,
}

#[derive(Clone, Serialize)]
struct Config {
    host: String,
    port: u16,
    timeout: u64,
    reverse_proxy: bool,
    proxy: ProxyConfig,
    swagger: bool,
    search_by_default: bool,
    third_party: ThirdPartyConfig,
}

#[derive(Clone, Serialize)]
struct ProxyConfig {
    enabled: bool,
    img_compress: bool,
}

#[derive(Clone, Serialize)]
struct ThirdPartyConfig {
    searx_url: Option<String>,
    webder_url: Option<String>,
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
    let config = Config::from_env();
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
        .nest_service("/static", ServeDir::new("packages/server/dist/static"))
        .with_state(AppState { config });

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("bind txtdot v2 server");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .expect("run txtdot v2 server");
}

impl Config {
    fn from_env() -> Self {
        Self {
            host: env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            port: env::var("PORT")
                .ok()
                .and_then(|value| value.parse().ok())
                .unwrap_or(8080),
            timeout: env::var("TIMEOUT")
                .ok()
                .and_then(|value| value.parse().ok())
                .unwrap_or(0),
            reverse_proxy: env_bool("REVERSE_PROXY", false),
            proxy: ProxyConfig {
                enabled: env_bool("PROXY_RES", true),
                img_compress: env_bool("IMG_COMPRESS", true),
            },
            swagger: env_bool("SWAGGER", false),
            search_by_default: env_bool("SEARCH_BY_DEFAULT", false),
            third_party: ThirdPartyConfig {
                searx_url: env::var("SEARX_URL").ok(),
                webder_url: env::var("WEBDER_URL").ok(),
            },
        }
    }
}

fn env_bool(key: &str, default: bool) -> bool {
    env::var(key)
        .map(|value| value == "true" || value == "1")
        .unwrap_or(default)
}

async fn shutdown_signal() {
    let _ = tokio::signal::ctrl_c().await;
}

async fn index(State(state): State<AppState>) -> Html<String> {
    Html(format!(
        r#"<!DOCTYPE html>
<html>
  <head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <meta http-equiv="X-UA-Compatible" content="IE=edge">
    <meta name="description" content="{description}">
    <title>txt. main page</title>
    <link rel="stylesheet" href="/static/common.css">
    <link rel="stylesheet" href="/static/index.css">
    <link rel="stylesheet" href="/static/form.css">
    <link rel="stylesheet" href="/static/form-inputs.css">
  </head>
  <body>
    <main>
      <header>
        <h1>txt<span class="dot">.</span></h1>
        <div class="menu">
          <a href="https://github.com/TxtDot/txtdot/releases/tag/v{version}" class="button secondary">v{version}</a>
          <a href="https://github.com/txtdot/txtdot" class="button secondary">GitHub</a>
          <a href="https://txtdot.github.io/documentation" class="button secondary">Docs</a>
          <a href="/configuration" class="button secondary">Configuration</a>
        </div>
        <p>{description}</p>
      </header>
      {form}
    </main>
  </body>
</html>"#,
        description = escape_html(description()),
        version = env!("CARGO_PKG_VERSION"),
        form = main_form(state.config.third_party.searx_url.is_some()),
    ))
}

async fn get_page(Query(params): Query<GetParams>) -> Result<Response, AppError> {
    let page = requester().process(&params.url).await?;
    let format = params.format.as_deref().unwrap_or("html");

    if format == "dalet" {
        return Ok(Json(ApiResponse {
            data: page,
            error: None,
        })
        .into_response());
    }

    let content = render_output("text/html", page)?;
    Ok(Html(page_shell(
        "txt. parsed page",
        &params.url,
        content.as_str(),
    ))
    .into_response())
}

async fn configuration(State(state): State<AppState>) -> Html<String> {
    let routes = routes()
        .iter()
        .map(|route| format!(r#"<a class="button secondary" href="{route}">{route}</a>"#))
        .collect::<Vec<_>>()
        .join("");

    Html(format!(
        r#"<!DOCTYPE html>
<html>
  <head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <meta http-equiv="X-UA-Compatible" content="IE=edge">
    <meta name="description" content="{description}">
    <title>txt. configuration</title>
    <link rel="stylesheet" href="/static/common.css">
    <link rel="stylesheet" href="/static/configuration.css">
  </head>
  <body>
    <main>
      <header>
        <h1>txt<span class="dot">.</span></h1>
        <div class="menu"><a class="button secondary" href="/">Home</a></div>
        <p>{description}</p>
      </header>
      <div class="configuration">
        <h2>Configuration</h2>
        <pre>{config}</pre>
        <h2>Available protocols</h2>
        <ol><li>http/https</li><li>gemini</li><li>gopher</li></ol>
        <h2>Available inputs</h2>
        <ol><li>application/daletpack</li><li>text/gemini</li><li>text/x-gophermap</li><li>text/markdown</li><li>text/x-markdown</li><li>text/html</li><li>text/plain</li><li>text/*</li></ol>
        <h2>Available outputs</h2>
        <ol><li>dalet</li><li>text/html</li><li>application/daletpack</li></ol>
        <h2>Available routes</h2>
        {routes}
      </div>
    </main>
  </body>
</html>"#,
        description = escape_html(description()),
        config = escape_html(&serde_json::to_string_pretty(&state.config).unwrap_or_default()),
        routes = routes,
    ))
}

async fn configuration_json(State(state): State<AppState>) -> Json<Value> {
    Json(json!({
        "version": env!("CARGO_PKG_VERSION"),
        "config": state.config,
        "protocols": ["http", "https", "gemini", "gopher"],
        "inputs": ["application/daletpack", "text/gemini", "text/x-gophermap", "text/markdown", "text/x-markdown", "text/html", "text/plain", "text/*"],
        "outputs": ["dalet", "text/html", "application/daletpack"],
        "routes": routes(),
    }))
}

async fn api_parse(Query(params): Query<GetParams>) -> Result<Json<Value>, AppError> {
    let page = requester().process(&params.url).await?;
    let format = params.format.as_deref().unwrap_or("dalet");
    let data = output_json(format, page)?;

    Ok(Json(json!(ApiResponse { data, error: None })))
}

async fn api_raw_html(Query(params): Query<GetParams>) -> Result<Html<String>, AppError> {
    let page = requester().process(&params.url).await?;
    Ok(Html(render_output("text/html", page)?))
}

async fn api_convert(Json(request): Json<ConvertRequest>) -> Result<Response, AppError> {
    let requester = requester();
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

    let output_type = request.output_type.as_deref().unwrap_or("dalet");
    if output_type == "text/html" || output_type == "html" {
        let html = render_output("text/html", page)?;
        return Ok(Html(html).into_response());
    }

    if output_type == "application/daletpack" {
        if let OutputData::Bytes(bytes) =
            requester.process_output("application/daletpack", page.clone())?
        {
            let mut headers = HeaderMap::new();
            headers.insert(
                header::CONTENT_TYPE,
                HeaderValue::from_static("application/daletpack"),
            );
            return Ok((headers, bytes).into_response());
        }
    }

    Ok(Json(ApiResponse {
        data: page,
        error: None,
    })
    .into_response())
}

fn requester() -> Requester<'static> {
    RequesterBuilder::default().plugin(requester_plugins).build()
}

fn render_output(output_type: &str, page: Page) -> Result<String, AppError> {
    match requester().process_output(output_type, page)? {
        OutputData::Text(text) => Ok(text),
        OutputData::Bytes(bytes) => {
            String::from_utf8(bytes).map_err(|error| AppError::BadRequest(error.to_string()))
        }
    }
}

fn output_json(format: &str, page: Page) -> Result<Value, AppError> {
    match format {
        "html" | "text/html" => Ok(json!({
            "output_type": "text/html",
            "content": render_output("text/html", page)?,
        })),
        "application/daletpack" | "daletpack" => {
            let OutputData::Bytes(bytes) = requester().process_output("application/daletpack", page)? else {
                return Err(AppError::BadRequest("daletpack output returned text".to_string()));
            };
            Ok(json!({
                "output_type": "application/daletpack",
                "content_base64": base64::engine::general_purpose::STANDARD.encode(bytes),
            }))
        }
        "dalet" | "json" => Ok(json!({
            "output_type": "dalet",
            "page": page,
        })),
        other => Err(AppError::BadRequest(format!("unsupported output format: {other}"))),
    }
}

fn main_form(search: bool) -> String {
    let search_form = if search {
        r#"<input type="checkbox" id="switch-search">
<label for="switch-search" class="switch-label">
  <span>URL</span>
  <span class="switch-btn"></span>
  <span>Search</span>
</label>
<form action="/search" method="get" class="input-grid main-form-search">
  <div class="input"><input type="text" name="q" placeholder="Search"></div>
  <div class="input"><input type="submit" class="button" value="Search"></div>
</form>"#
    } else {
        ""
    };

    format!(
        r#"{search_form}
<form action="/get" method="get" class="input-grid {search_class}">
  <div class="input">
    <input type="text" name="url" id="url" placeholder="URL" autofocus>
  </div>
  <div class="input">
    <input type="submit" id="submit" class="button" value="Parse">
  </div>
  <div class="input-row">
    <div class="input">
      <label for="format">Format</label>
      <select name="format">
        <option value="html" selected>HTML</option>
        <option value="dalet">Dalet</option>
      </select>
    </div>
  </div>
</form>"#,
        search_form = search_form,
        search_class = if search { "main-form-url" } else { "" },
    )
}

fn page_shell(title: &str, remote_url: &str, content: &str) -> String {
    format!(
        r#"<!doctype html>
<html lang="en">
  <head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <meta http-equiv="X-UA-Compatible" content="IE=edge" />
    <meta name="robots" content="noindex, nofollow" />
    <title>{title}</title>
    <link rel="stylesheet" href="/static/common.css" />
    <link rel="stylesheet" href="/static/get.css" />
  </head>
  <body>
    <main>
      <div class="menu">
        <a class="button secondary" href="/">Home</a>
        <a class="button secondary" href="{remote_url}">Original page</a>
      </div>
      <p class="title">{title}</p>
      {content}
    </main>
  </body>
</html>"#,
        title = escape_html(title),
        remote_url = escape_html(remote_url),
        content = content,
    )
}

fn routes() -> Vec<&'static str> {
    vec![
        "/",
        "/get",
        "/configuration",
        "/configuration/json",
        "/api/parse",
        "/api/raw-html",
        "/api/convert",
    ]
}

fn description() -> &'static str {
    "HTTP proxy that parses only text, links and pictures from pages"
}

fn escape_html(input: &str) -> String {
    let mut escaped = String::new();
    for ch in input.chars() {
        match ch {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&#39;"),
            _ => escaped.push(ch),
        }
    }
    escaped
}

enum AppError {
    Drova(drova_sdk::requester::Error),
    BadRequest(String),
}

impl From<drova_sdk::requester::Error> for AppError {
    fn from(error: drova_sdk::requester::Error) -> Self {
        Self::Drova(error)
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::Drova(error) => (StatusCode::BAD_GATEWAY, format!("{error:?}")),
            AppError::BadRequest(message) => (StatusCode::BAD_REQUEST, message),
        };

        (
            status,
            Json(json!({
                "data": null,
                "error": message,
            })),
        )
            .into_response()
    }
}
