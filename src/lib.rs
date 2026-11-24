pub mod api;
pub mod domain;
pub mod schema;

use diesel::{ConnectionError, ConnectionResult};
use diesel_async::{
	AsyncPgConnection,
	pooled_connection::{
		AsyncDieselConnectionManager, ManagerConfig, deadpool::Pool,
	},
};
use rustls::ClientConfig;
use rustls_platform_verifier::ConfigVerifierExt;
use tokio_postgres_rustls::MakeRustlsConnect;

/// Builds the connection pool and checks that the database is reachable.
///
/// The pool keeps connections open and lends them out: `pool.get()` hands out
/// an idle connection if there is one, and dropping it gives it back. Only when
/// no connection is idle does the pool open a new one by calling `connect`.
pub async fn build_pool(
	db_url: String,
) -> Result<Pool<AsyncPgConnection>, Box<dyn std::error::Error>> {
	// By default the pool would open plain connections. Have it call `connect`
	// instead, which adds TLS.
	let mut config = ManagerConfig::default();
	config.custom_setup = Box::new(|url| Box::pin(connect(url)));

	let manager =
		AsyncDieselConnectionManager::<AsyncPgConnection>::new_with_config(
			db_url, config,
		);
	// Building the pool doesn't connect to anything yet.
	let pool = Pool::builder(manager).build()?;

	// Open one connection now so that a wrong URL or a database that's down
	// stops the program at startup, instead of failing on the first request.
	// Dropping it returns it to the pool, where the first request reuses it.
	drop(pool.get().await?);
	Ok(pool)
}

/// Opens one new database connection, using TLS when the server offers it.
/// The pool calls this only when it has no idle connection to reuse.
///
/// `sslmode` in the URL decides whether TLS is required: `require` refuses to
/// connect without it (use this in production), while the default, `prefer`,
/// falls back to an unencrypted connection, which is what a local Postgres
/// without TLS gets.
async fn connect(url: &str) -> ConnectionResult<AsyncPgConnection> {
	// Trust the same certificate authorities as the operating system.
	let tls_config = ClientConfig::with_platform_verifier()
		.map_err(|e| ConnectionError::BadConnection(e.to_string()))?;
	let tls = MakeRustlsConnect::new(tls_config);

	// tokio-postgres returns the client and a background task that does the
	// network I/O; diesel-async takes both and runs the task for us.
	let (client, connection) = tokio_postgres::connect(url, tls)
		.await
		.map_err(|e| ConnectionError::BadConnection(e.to_string()))?;
	AsyncPgConnection::try_from_client_and_connection(client, connection).await
}
