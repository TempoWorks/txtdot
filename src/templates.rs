use tera::{Context, Tera};

use crate::error::AppError;

pub fn render(path: &str, context: &Context) -> Result<String, AppError> {
    let tera = Tera::new("templates/**/*.tera")
        .map_err(|error| AppError::BadRequest(error.to_string()))?;

    tera.render(path, context)
        .map_err(|error| AppError::BadRequest(error.to_string()))
}
