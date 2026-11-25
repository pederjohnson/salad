//! The HTTP layer. Handlers here translate requests into calls to
//! `crate::domain`, which holds all the database logic.

mod auth;
mod error;
mod extract;
mod members;

use axum::{Router, routing::get};
use diesel_async::{
	AsyncPgConnection, RunQueryDsl, pooled_connection::deadpool::Pool,
};
use tower_http::trace::TraceLayer;

pub use error::AppError;
pub use extract::{AuthUser, Db, RequireOfficer};

#[derive(Clone)]
pub struct AppState {
	pub pool: Pool<AsyncPgConnection>,
}

pub fn router(state: AppState) -> Router {
	Router::new()
		.route("/api/health", get(health))
		.nest("/api/auth", auth::router())
		.nest("/api/members", members::router())
		.layer(TraceLayer::new_for_http())
		.with_state(state)
}

/// Responds "ok" if the server can reach the database.
async fn health(Db(mut conn): Db) -> Result<&'static str, AppError> {
	diesel::sql_query("SELECT 1").execute(&mut conn).await?;
	Ok("ok")
}
