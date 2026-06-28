use std::{
    net::{IpAddr, SocketAddr},
    sync::Arc,
};

use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

mod config;
mod drova;
mod error;
mod proxy;
mod routes;
mod state;
mod templates;

use config::Config;
use state::AppState;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    let log_filter = std::env::var("RUST_LOG")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| "info".to_string());
    tracing_subscriber::registry()
        .with(EnvFilter::new(log_filter))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = Arc::new(Config::from_env());
    let configured_host = config.host.clone();
    let configured_port = config.port;
    let addr = SocketAddr::from((
        config.host.parse::<IpAddr>().unwrap_or_else(|error| {
            tracing::warn!(
                host = %config.host,
                %error,
                "invalid HOST, falling back to 0.0.0.0"
            );
            [0, 0, 0, 0].into()
        }),
        config.port,
    ));
    let static_dir = std::env::var("TXTDOT_STATIC_DIR")
        .unwrap_or_else(|_| env!("TXTDOT_BUILD_STATIC_DIR").to_string());
    tracing::info!(
        host = %configured_host,
        port = configured_port,
        bind = %addr,
        static_dir = %static_dir,
        "starting txtdot v2 server"
    );

    let app = routes::router(AppState {
        config,
        routes: Arc::new(routes::app_routes()),
    })
    .nest_service("/static", ServeDir::new(static_dir))
    .layer(TraceLayer::new_for_http());

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("bind txtdot v2 server");
    let local_addr = listener
        .local_addr()
        .expect("read txtdot v2 server listener address");
    tracing::info!(
        ip = %local_addr.ip(),
        port = local_addr.port(),
        addr = %local_addr,
        "txtdot v2 server listening"
    );

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .expect("run txtdot v2 server");
}

async fn shutdown_signal() {
    let _ = tokio::signal::ctrl_c().await;
}
