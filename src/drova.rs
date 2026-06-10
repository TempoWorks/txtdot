use axum::{
    http::{header, HeaderMap, HeaderValue},
    response::{IntoResponse, Response},
};
use dalet::types::Page;
use drova_plugins::requester_plugins;
use drova_sdk::requester::{OutputData, Requester, RequesterBuilder};
use serde::Serialize;

use crate::error::AppError;

pub const HTML_OUTPUT: &str = "text/html";
pub const DALETPACK_OUTPUT: &str = "application/daletpack";
pub const DEFAULT_PAGE_OUTPUT: &str = HTML_OUTPUT;
pub const DEFAULT_API_OUTPUT: &str = DALETPACK_OUTPUT;

#[derive(Serialize)]
pub struct OutputOption {
    value: String,
    label: String,
    selected: bool,
}

pub fn requester() -> Requester<'static> {
    RequesterBuilder::default()
        .plugin(requester_plugins)
        .build()
}

pub fn supported_protocols() -> Vec<String> {
    requester().protocols().map(str::to_string).collect()
}

pub fn supported_inputs() -> Vec<String> {
    requester().inputs().map(str::to_string).collect()
}

pub fn supported_outputs() -> Vec<String> {
    requester().outputs().map(str::to_string).collect()
}

pub fn output_options(selected: &str) -> Vec<OutputOption> {
    supported_outputs()
        .into_iter()
        .map(|value| OutputOption {
            selected: value == selected,
            label: output_label(&value),
            value,
        })
        .collect()
}

pub fn resolve_output_type(output_type: &str) -> Result<String, AppError> {
    let output_type = match output_type {
        "html" => HTML_OUTPUT,
        "dalet" | "daletpack" => DALETPACK_OUTPUT,
        other => other,
    };

    if supported_outputs()
        .iter()
        .any(|output| output == output_type)
    {
        Ok(output_type.to_string())
    } else {
        Err(AppError::BadRequest(format!(
            "unsupported output format: {output_type}"
        )))
    }
}

pub fn is_html_output(output_type: &str) -> bool {
    output_type == HTML_OUTPUT
}

pub fn is_daletpack_output(output_type: &str) -> bool {
    output_type == DALETPACK_OUTPUT
}

pub fn render_html(page: Page) -> Result<String, AppError> {
    match requester().process_output(HTML_OUTPUT, page)? {
        OutputData::Text(text) => Ok(text),
        OutputData::Bytes(bytes) => {
            String::from_utf8(bytes).map_err(|error| AppError::BadRequest(error.to_string()))
        }
    }
}

pub fn render_daletpack(page: Page) -> Result<Vec<u8>, AppError> {
    match requester().process_output(DALETPACK_OUTPUT, page)? {
        OutputData::Bytes(bytes) => Ok(bytes),
        OutputData::Text(_) => Err(AppError::BadRequest(
            "daletpack output returned text".to_string(),
        )),
    }
}

pub fn daletpack_response(bytes: Result<Vec<u8>, AppError>) -> Result<Response, AppError> {
    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static(DALETPACK_OUTPUT),
    );
    headers.insert(
        header::CONTENT_DISPOSITION,
        HeaderValue::from_static("attachment; filename=\"txtdot.daletpack\""),
    );

    Ok((headers, bytes?).into_response())
}

pub fn output_response(output_type: &str, page: Page) -> Result<Response, AppError> {
    if is_daletpack_output(output_type) {
        return daletpack_response(render_daletpack(page));
    }

    let content_type = HeaderValue::from_str(output_type)
        .map_err(|error| AppError::BadRequest(error.to_string()))?;
    let mut headers = HeaderMap::new();
    headers.insert(header::CONTENT_TYPE, content_type);

    match requester().process_output(output_type, page)? {
        OutputData::Text(text) => Ok((headers, text).into_response()),
        OutputData::Bytes(bytes) => Ok((headers, bytes).into_response()),
    }
}

fn output_label(output_type: &str) -> String {
    let name = output_type.rsplit('/').next().unwrap_or(output_type);
    if name.len() <= 4 {
        name.to_uppercase()
    } else {
        let mut chars = name.chars();
        match chars.next() {
            Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            None => String::new(),
        }
    }
}
