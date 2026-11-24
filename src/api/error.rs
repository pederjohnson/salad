use axum::{
	Json,
	http::StatusCode,
	response::{IntoResponse, Response},
};
use serde_json::json;

/// Every handler returns this on failure. Client errors carry a message the
/// user can see; internal errors are logged and reported generically so no
/// database details leak to the client.
#[derive(Debug)]
pub enum AppError {
	BadRequest(&'static str),
	Unauthorized,
	Internal(String),
}

impl IntoResponse for AppError {
	fn into_response(self) -> Response {
		let (status, message) = match self {
			AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
			AppError::Unauthorized => {
				(StatusCode::UNAUTHORIZED, "Not signed in, or sign-in failed")
			}
			AppError::Internal(detail) => {
				tracing::error!("internal error: {detail}");
				(StatusCode::INTERNAL_SERVER_ERROR, "Something went wrong")
			}
		};
		(status, Json(json!({ "error": message }))).into_response()
	}
}

impl From<diesel::result::Error> for AppError {
	fn from(e: diesel::result::Error) -> Self {
		AppError::Internal(format!("database: {e}"))
	}
}

impl From<diesel_async::pooled_connection::deadpool::PoolError> for AppError {
	fn from(e: diesel_async::pooled_connection::deadpool::PoolError) -> Self {
		AppError::Internal(format!("connection pool: {e}"))
	}
}
