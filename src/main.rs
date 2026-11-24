use diesel_async::{
	AsyncPgConnection,
	pooled_connection::{AsyncDieselConnectionManager, deadpool::Pool},
};
use salad::api::{self, AppState};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	dotenvy::dotenv().ok();
	tracing_subscriber::fmt()
		.with_env_filter(
			EnvFilter::try_from_default_env()
				.unwrap_or_else(|_| "salad=debug,tower_http=info".into()),
		)
		.init();

	let db_url = std::env::var("DATABASE_URL")
		.expect("Environment variables not configured!");
	let manager =
		AsyncDieselConnectionManager::<AsyncPgConnection>::new(db_url);
	let pool = Pool::builder(manager).build()?;

	// Fail at startup, not on the first request, if the database is down.
	drop(pool.get().await?);

	if cfg!(debug_assertions) {
		tracing::warn!(
			"debug build: any 6-digit sign-in code is accepted. \
			 Never deploy a debug build."
		);
	}

	let addr = std::env::var("BIND_ADDR")
		.unwrap_or_else(|_| "127.0.0.1:8000".to_string());
	let listener = tokio::net::TcpListener::bind(&addr).await?;
	tracing::info!("listening on http://{addr}");

	axum::serve(listener, api::router(AppState { pool })).await?;
	Ok(())
}
