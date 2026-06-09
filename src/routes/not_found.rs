use crate::error::AppError;

pub async fn not_found() -> AppError {
    AppError::NotFound("route not found".to_string())
}
