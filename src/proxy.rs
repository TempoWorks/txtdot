use axum::{
    extract::Query,
    http::{header, HeaderMap, HeaderValue},
    response::{IntoResponse, Response},
};
use image::DynamicImage;
use serde::Deserialize;
use url::Url;

use crate::error::AppError;

#[derive(Deserialize)]
pub struct ProxyParams {
    pub url: String,
}

pub async fn proxy(Query(params): Query<ProxyParams>) -> Result<Response, AppError> {
    let response = reqwest::get(&params.url)
        .await
        .map_err(|error| AppError::Upstream(error.to_string()))?;
    let headers = response.headers().clone();
    let bytes = response
        .bytes()
        .await
        .map_err(|error| AppError::Upstream(error.to_string()))?;

    let mut out = HeaderMap::new();
    if let Some(content_type) = headers.get(header::CONTENT_TYPE) {
        out.insert(header::CONTENT_TYPE, content_type.clone());
    }
    out.insert(
        header::CONTENT_LENGTH,
        HeaderValue::from_str(&bytes.len().to_string())
            .map_err(|error| AppError::Upstream(error.to_string()))?,
    );

    Ok((out, bytes).into_response())
}

pub async fn proxy_img(Query(params): Query<ProxyParams>) -> Result<Response, AppError> {
    let response = reqwest::get(&params.url)
        .await
        .map_err(|error| AppError::Upstream(error.to_string()))?;
    let mime = response
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .to_string();
    let bytes = response
        .bytes()
        .await
        .map_err(|error| AppError::Upstream(error.to_string()))?;

    if mime.starts_with("image/svg") {
        let mut headers = HeaderMap::new();
        headers.insert(header::CONTENT_TYPE, HeaderValue::from_static("image/svg+xml"));
        headers.insert(
            header::CONTENT_LENGTH,
            HeaderValue::from_str(&bytes.len().to_string())
                .map_err(|error| AppError::Upstream(error.to_string()))?,
        );
        return Ok((headers, bytes).into_response());
    }

    let image = image::load_from_memory(&bytes)
        .map_err(|error| AppError::Upstream(error.to_string()))?;
    let compressed = encode_webp(&image)?;

    let mut headers = HeaderMap::new();
    headers.insert(header::CONTENT_TYPE, HeaderValue::from_static("image/webp"));
    headers.insert(
        header::CONTENT_LENGTH,
        HeaderValue::from_str(&compressed.len().to_string())
            .map_err(|error| AppError::Upstream(error.to_string()))?,
    );
    headers.insert(
        "x-original-size",
        HeaderValue::from_str(&bytes.len().to_string())
            .map_err(|error| AppError::Upstream(error.to_string()))?,
    );
    headers.insert(
        "x-bytes-saved",
        HeaderValue::from_str(&(bytes.len() as isize - compressed.len() as isize).to_string())
            .map_err(|error| AppError::Upstream(error.to_string()))?,
    );

    Ok((headers, compressed).into_response())
}

pub fn rewrite_html_links(
    html: &str,
    request_base: &str,
    remote_url: &str,
    img_compress: bool,
) -> Result<String, AppError> {
    use lol_html::{element, rewrite_str, RewriteStrSettings};

    let remote = Url::parse(remote_url).map_err(|error| AppError::BadRequest(error.to_string()))?;
    let request =
        Url::parse(request_base).map_err(|error| AppError::BadRequest(error.to_string()))?;

    let parser_url = |href: &str| parser_url(&request, &remote, href);
    let proxied_url = |href: &str| proxy_url(&request, &remote, href, false);
    let img_proxy_url = |href: &str| proxy_url(&request, &remote, href, img_compress);

    rewrite_str(
        html,
        RewriteStrSettings::new()
            .append_element_content_handler(element!("a[href]", move |el| {
                rewrite_attr(el, "href", &parser_url);
                Ok(())
            }))
            .append_element_content_handler(element!("frame[src], iframe[src]", move |el| {
                rewrite_attr(el, "src", &parser_url);
                Ok(())
            }))
            .append_element_content_handler(element!("video[src], audio[src], embed[src], track[src], source[src]", move |el| {
                rewrite_attr(el, "src", &proxied_url);
                Ok(())
            }))
            .append_element_content_handler(element!("object[data]", move |el| {
                rewrite_attr(el, "data", &proxied_url);
                Ok(())
            }))
            .append_element_content_handler(element!("img[src], image[src]", move |el| {
                rewrite_attr(el, "src", &img_proxy_url);
                Ok(())
            }))
            .append_element_content_handler(element!("source[srcset], img[srcset]", move |el| {
                if let Some(srcset) = el.get_attribute("srcset") {
                    let rewritten = srcset
                        .split(',')
                        .map(|candidate| rewrite_srcset_candidate(candidate, &proxied_url))
                        .collect::<Vec<_>>()
                        .join(", ");
                    let _ = el.set_attribute("srcset", &rewritten);
                }
                Ok(())
            })),
    )
    .map_err(|error| AppError::BadRequest(error.to_string()))
}

fn rewrite_attr<F>(el: &mut lol_html::html_content::Element, attr: &str, build: &F)
where
    F: Fn(&str) -> Option<String>,
{
    if let Some(value) = el.get_attribute(attr).and_then(|value| build(&value)) {
        let _ = el.set_attribute(attr, &value);
    }
}

fn rewrite_srcset_candidate<F>(candidate: &str, build: &F) -> String
where
    F: Fn(&str) -> Option<String>,
{
    let mut parts = candidate.split_whitespace().collect::<Vec<_>>();
    if let Some(first) = parts.first_mut() {
        if let Some(value) = build(first) {
            *first = Box::leak(value.into_boxed_str());
        }
    }
    parts.join(" ")
}

fn parser_url(request: &Url, remote: &Url, href: &str) -> Option<String> {
    let resolved = remote.join(href).ok()?;
    let mut url = request.join("/get").ok()?;
    url.query_pairs_mut().append_pair("url", resolved.as_str());
    Some(url.to_string())
}

fn proxy_url(request: &Url, remote: &Url, href: &str, img: bool) -> Option<String> {
    let resolved = remote.join(href).ok()?;
    let mut url = request.join(if img { "/proxy/img" } else { "/proxy" }).ok()?;
    url.query_pairs_mut().append_pair("url", resolved.as_str());
    Some(url.to_string())
}

fn encode_webp(image: &DynamicImage) -> Result<Vec<u8>, AppError> {
    let rgba = DynamicImage::ImageRgba8(image.to_rgba8());
    let encoder =
        webp::Encoder::from_image(&rgba).map_err(|error| AppError::Upstream(error.to_string()))?;
    Ok(encoder.encode(25.0).to_vec())
}
