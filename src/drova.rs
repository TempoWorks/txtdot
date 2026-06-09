use dalet::types::Page;
use drova_plugins::requester_plugins;
use drova_sdk::requester::{OutputData, Requester, RequesterBuilder};

use crate::error::AppError;

pub fn requester() -> Requester<'static> {
    RequesterBuilder::default().plugin(requester_plugins).build()
}

pub fn render_html(page: Page) -> Result<String, AppError> {
    match requester().process_output("text/html", page)? {
        OutputData::Text(text) => Ok(text),
        OutputData::Bytes(bytes) => {
            String::from_utf8(bytes).map_err(|error| AppError::BadRequest(error.to_string()))
        }
    }
}

pub fn render_daletpack(page: Page) -> Result<Vec<u8>, AppError> {
    match requester().process_output("application/daletpack", page)? {
        OutputData::Bytes(bytes) => Ok(bytes),
        OutputData::Text(_) => Err(AppError::BadRequest(
            "daletpack output returned text".to_string(),
        )),
    }
}
