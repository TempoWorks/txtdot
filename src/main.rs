use std::{
    net::{IpAddr, SocketAddr},
    sync::Arc,
};

use tower_http::services::ServeDir;

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

    let config = Arc::new(Config::from_env());
    let addr = SocketAddr::from((
        config
            .host
            .parse::<IpAddr>()
            .unwrap_or_else(|_| [0, 0, 0, 0].into()),
        config.port,
    ));
    let static_dir = std::env::var("TXTDOT_STATIC_DIR")
        .unwrap_or_else(|_| env!("TXTDOT_BUILD_STATIC_DIR").to_string());

    let app = routes::router(AppState {
        config,
        routes: Arc::new(routes::app_routes()),
    })
    .nest_service("/static", ServeDir::new(static_dir));

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
