use axum::{
	Json,
	http::StatusCode,
	response::{IntoResponse, Response},
};
use diesel::result::{DatabaseErrorKind, Error as DieselError};
use serde_json::json;

/// Every handler returns this on failure. Client errors carry a message the
/// user can see; internal errors are logged and reported generically so no
/// database details leak to the client.
#[derive(Debug)]
pub enum AppError {
	BadRequest(&'static str),
	Unauthorized,
	/// Signed in, but not allowed to do this.
	Forbidden,
	NotFound(&'static str),
	/// The request clashes with existing data, such as a NetID already in use.
	Conflict(&'static str),
	Internal(String),
}

impl IntoResponse for AppError {
	fn into_response(self) -> Response {
		let (status, message) = match self {
			AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
			AppError::Unauthorized => {
				(StatusCode::UNAUTHORIZED, "Not signed in, or sign-in failed")
			}
			AppError::Forbidden => (
				StatusCode::FORBIDDEN,
				"You don't have permission to do that",
			),
			AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
			AppError::Conflict(msg) => (StatusCode::CONFLICT, msg),
			AppError::Internal(detail) => {
				tracing::error!("internal error: {detail}");
				(StatusCode::INTERNAL_SERVER_ERROR, "Something went wrong")
			}
		};
		(status, Json(json!({ "error": message }))).into_response()
	}
}

/// Breaking one of the database's rules (a UNIQUE, CHECK, foreign key or NOT
/// NULL constraint) is the client's fault, so it becomes a 409 or 400 with a
/// message from `constraint_message`. Anything else is a 500.
impl From<DieselError> for AppError {
	fn from(e: DieselError) -> Self {
		if let DieselError::DatabaseError(kind, info) = &e {
			let constraint = info.constraint_name().unwrap_or_default();
			match kind {
				DatabaseErrorKind::UniqueViolation => {
					return AppError::Conflict(constraint_message(constraint));
				}
				DatabaseErrorKind::CheckViolation
				| DatabaseErrorKind::ForeignKeyViolation
				| DatabaseErrorKind::NotNullViolation => {
					return AppError::BadRequest(constraint_message(
						constraint,
					));
				}
				_ => {}
			}
		}
		AppError::Internal(format!("database: {e}"))
	}
}

/// What to tell the user when a request breaks the database rule `name`.
/// The names are Postgres's defaults for the constraints in the migrations
/// (`<table>_<column>_key` for UNIQUE, `_check`, `_fkey`, `_pkey`). Add a line
/// here when a new constraint can be broken by user input.
fn constraint_message(name: &str) -> &'static str {
	match name {
		"members_pkey" => "Another member already has that NetID",
		"members_net_id_check" => "NetID should look like abc123456",
		"members_card_id_key" => "Another member already has that card number",
		"members_card_id_check" => "The card number should be digits only",
		"members_discord_id_key" => {
			"Another member already has that Discord account"
		}
		"members_discord_id_check" => "The Discord ID should be digits only",
		"members_major_fkey" => "No such major", // TO DO: auto-add major?
		"staff_project_fkey" => "No such project", // TO DO: frontend staff table
		_ => {
			tracing::warn!("no message for database constraint {name:?}");
			"That doesn't fit the existing data"
		}
	}
}

impl From<diesel_async::pooled_connection::deadpool::PoolError> for AppError {
	fn from(e: diesel_async::pooled_connection::deadpool::PoolError) -> Self {
		AppError::Internal(format!("connection pool: {e}"))
	}
}
