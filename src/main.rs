use diesel::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	dotenvy::dotenv().ok();
	let db_url = std::env::var("DATABASE_URL")
		.expect("Environment variables not configured!");
	let mut _connection = PgConnection::establish(&db_url)?;

	println!("connected");

	Ok(())
}
