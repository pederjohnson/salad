//! The HTTP layer. Handlers here translate requests into calls to
//! `crate::domain`, which holds all the database logic.

mod auth;
mod error;

use axum::{
	Router, extract::FromRequestParts, http::request::Parts, routing::get,
};
use diesel_async::{
	AsyncPgConnection, RunQueryDsl,
	pooled_connection::deadpool::{Object, Pool},
};
use tower_http::trace::TraceLayer;

pub use auth::AuthUser;
pub use error::AppError;

#[derive(Clone)]
pub struct AppState {
	pub pool: Pool<AsyncPgConnection>,
}

/// A database connection from the pool, returned when the handler finishes.
/// Take it as a handler argument, `Db(mut conn): Db`, and pass `&mut conn`
/// to `crate::domain` functions.
pub struct Db(pub Object<AsyncPgConnection>);

impl FromRequestParts<AppState> for Db {
	type Rejection = AppError;

	async fn from_request_parts(
		_parts: &mut Parts,
		state: &AppState,
	) -> Result<Self, Self::Rejection> {
		Ok(Db(state.pool.get().await?))
	}
}

pub fn router(state: AppState) -> Router {
	Router::new()
		.route("/api/health", get(health))
		.nest("/api/auth", auth::router())
		.layer(TraceLayer::new_for_http())
		.with_state(state)
}

/// Responds "ok" if the server can reach the database.
async fn health(Db(mut conn): Db) -> Result<&'static str, AppError> {
	diesel::sql_query("SELECT 1").execute(&mut conn).await?;
	Ok("ok")
}
