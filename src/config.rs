use std::env;

use serde::Serialize;

#[derive(Clone, Serialize)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub timeout: u64,
    pub reverse_proxy: bool,
    pub proxy: ProxyConfig,
    pub swagger: bool,
    pub search_by_default: bool,
    pub third_party: ThirdPartyConfig,
}

pub const DESCRIPTION: &str = "HTTP proxy that parses only text, links and pictures from pages";

#[derive(Clone, Serialize)]
pub struct ProxyConfig {
    pub enabled: bool,
    pub img_compress: bool,
}

#[derive(Clone, Serialize)]
pub struct ThirdPartyConfig {
    pub searx_url: Option<String>,
}

impl Config {
    pub fn from_env() -> Self {
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
                searx_url: env::var("SEARX_URL")
                    .ok()
                    .filter(|value| !value.trim().is_empty()),
            },
        }
    }
}

fn env_bool(key: &str, default: bool) -> bool {
    env::var(key)
        .map(|value| value == "true" || value == "1")
        .unwrap_or(default)
}
