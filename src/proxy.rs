use axum::{
    extract::Query,
    http::{header, HeaderMap, HeaderValue},
    response::{IntoResponse, Response},
};
use image::{DynamicImage, ImageEncoder};
use serde::Deserialize;
use url::Url;

use crate::error::AppError;

#[derive(Clone, Copy)]
pub struct RewriteOptions {
    pub proxy_documents: bool,
    pub proxy_images: bool,
    pub proxy_media: bool,
    pub proxy_files: bool,
    pub process_images: bool,
}

#[derive(Deserialize)]
pub struct ProxyParams {
    pub url: String,
}

#[derive(Deserialize)]
pub struct ProxyImgParams {
    pub url: String,
    pub w: Option<u32>,
}

pub async fn proxy(Query(params): Query<ProxyParams>) -> Result<Response, AppError> {
    tracing::info!(url = %params.url, "proxy request started");
    let response = reqwest::get(&params.url)
        .await
        .map_err(|error| AppError::Upstream(error.to_string()))?;
    let status = response.status();
    let headers = response.headers().clone();
    let bytes = response
        .bytes()
        .await
        .map_err(|error| AppError::Upstream(error.to_string()))?;
    tracing::info!(
        url = %params.url,
        status = status.as_u16(),
        bytes = bytes.len(),
        "proxy request completed"
    );

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

pub async fn proxy_img(Query(params): Query<ProxyImgParams>) -> Result<Response, AppError> {
    tracing::info!(
        url = %params.url,
        requested_width = params.w,
        output_format = "avif",
        "image proxy request started"
    );
    let response = reqwest::get(&params.url)
        .await
        .map_err(|error| AppError::Upstream(error.to_string()))?;
    let status = response.status();
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
    tracing::info!(
        url = %params.url,
        status = status.as_u16(),
        content_type = %mime,
        original_bytes = bytes.len(),
        "image proxy upstream fetched"
    );

    if mime.starts_with("image/svg") {
        tracing::info!(
            url = %params.url,
            bytes = bytes.len(),
            "image proxy passed through svg"
        );
        let mut headers = HeaderMap::new();
        headers.insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("image/svg+xml"),
        );
        headers.insert(
            header::CONTENT_LENGTH,
            HeaderValue::from_str(&bytes.len().to_string())
                .map_err(|error| AppError::Upstream(error.to_string()))?,
        );
        return Ok((headers, bytes).into_response());
    }

    let mut image =
        image::load_from_memory(&bytes).map_err(|error| AppError::Upstream(error.to_string()))?;
    let original_width = image.width();
    let original_height = image.height();
    if let Some(width) = params
        .w
        .filter(|width| *width > 0 && *width < image.width())
    {
        let height = ((image.height() as u64 * width as u64) / image.width() as u64)
            .max(1)
            .min(u32::MAX as u64) as u32;
        image = image.resize_exact(width, height, image::imageops::FilterType::Lanczos3);
    }
    let compressed = encode_avif(&image)?;

    let mut headers = HeaderMap::new();
    headers.insert(header::CONTENT_TYPE, HeaderValue::from_static("image/avif"));
    headers.insert("x-image-format", HeaderValue::from_static("avif"));
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
    tracing::info!(
        url = %params.url,
        original_width,
        original_height,
        output_width = image.width(),
        output_height = image.height(),
        original_bytes = bytes.len(),
        avif_bytes = compressed.len(),
        bytes_saved = bytes.len() as isize - compressed.len() as isize,
        "image proxy encoded avif"
    );

    Ok((headers, compressed).into_response())
}

pub fn rewrite_html_links(
    html: &str,
    request_base: &str,
    remote_url: &str,
    options: RewriteOptions,
) -> Result<String, AppError> {
    use lol_html::{element, rewrite_str, RewriteStrSettings};

    let remote = Url::parse(remote_url).map_err(|error| AppError::BadRequest(error.to_string()))?;
    let request =
        Url::parse(request_base).map_err(|error| AppError::BadRequest(error.to_string()))?;

    let document_url = |href: &str| {
        if options.proxy_documents {
            parser_url(&request, &remote, href)
        } else {
            direct_url(&remote, href)
        }
    };
    let frame_url = |href: &str| {
        if options.proxy_documents {
            parser_url(&request, &remote, href)
        } else {
            direct_url(&remote, href)
        }
    };
    let media_url = |href: &str| {
        if options.proxy_media {
            proxy_url(&request, &remote, href, false)
        } else {
            direct_url(&remote, href)
        }
    };
    let file_url = |href: &str| {
        if options.proxy_files {
            proxy_url(&request, &remote, href, false)
        } else {
            direct_url(&remote, href)
        }
    };
    let img_url = |href: &str| {
        if options.proxy_images || options.process_images {
            proxy_url(&request, &remote, href, options.process_images)
        } else {
            direct_url(&remote, href)
        }
    };
    let img_srcset = |href: &str| responsive_srcset(&request, &remote, href);

    rewrite_str(
        html,
        RewriteStrSettings::new()
            .append_element_content_handler(element!("a[href]", move |el| {
                rewrite_attr(el, "href", &document_url);
                Ok(())
            }))
            .append_element_content_handler(element!("frame[src], iframe[src]", move |el| {
                rewrite_attr(el, "src", &frame_url);
                Ok(())
            }))
            .append_element_content_handler(element!(
                "video[src], audio[src], source[src]",
                move |el| {
                    rewrite_attr(el, "src", &media_url);
                    Ok(())
                }
            ))
            .append_element_content_handler(element!("embed[src], track[src]", move |el| {
                rewrite_attr(el, "src", &file_url);
                Ok(())
            }))
            .append_element_content_handler(element!("object[data]", move |el| {
                rewrite_attr(el, "data", &file_url);
                Ok(())
            }))
            .append_element_content_handler(element!("img[src], image[src]", move |el| {
                if options.process_images && el.get_attribute("srcset").is_none() {
                    if let Some(src) = el.get_attribute("src") {
                        if let Some(srcset) = img_srcset(&src) {
                            let _ = el.set_attribute("srcset", &srcset);
                            if el.get_attribute("sizes").is_none() {
                                let _ =
                                    el.set_attribute("sizes", "(max-width: 768px) 100vw, 768px");
                            }
                        }
                    }
                }
                rewrite_attr(el, "src", &img_url);
                Ok(())
            }))
            .append_element_content_handler(element!("source[srcset], img[srcset]", move |el| {
                if let Some(srcset) = el.get_attribute("srcset") {
                    let rewritten = srcset
                        .split(',')
                        .map(|candidate| rewrite_srcset_candidate(candidate, &img_url))
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

fn direct_url(remote: &Url, href: &str) -> Option<String> {
    remote.join(href).ok().map(|url| url.to_string())
}

fn proxy_url(request: &Url, remote: &Url, href: &str, img: bool) -> Option<String> {
    let resolved = remote.join(href).ok()?;
    let mut url = request
        .join(if img { "/proxy/img" } else { "/proxy" })
        .ok()?;
    url.query_pairs_mut().append_pair("url", resolved.as_str());
    Some(url.to_string())
}

fn responsive_srcset(request: &Url, remote: &Url, href: &str) -> Option<String> {
    const WIDTHS: [u32; 4] = [320, 640, 960, 1280];

    WIDTHS
        .iter()
        .map(|width| {
            let resolved = remote.join(href).ok()?;
            let mut url = request.join("/proxy/img").ok()?;
            url.query_pairs_mut()
                .append_pair("url", resolved.as_str())
                .append_pair("w", &width.to_string());
            Some(format!("{url} {width}w"))
        })
        .collect::<Option<Vec<_>>>()
        .map(|candidates| candidates.join(", "))
}

fn encode_avif(image: &DynamicImage) -> Result<Vec<u8>, AppError> {
    let rgba = image.to_rgba8();
    let mut compressed = Vec::new();
    image::codecs::avif::AvifEncoder::new_with_speed_quality(&mut compressed, 7, 55)
        .write_image(
            rgba.as_raw(),
            rgba.width(),
            rgba.height(),
            image::ExtendedColorType::Rgba8,
        )
        .map_err(|error| AppError::Upstream(error.to_string()))?;
    Ok(compressed)
}
