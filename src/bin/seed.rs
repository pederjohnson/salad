//! Loads test data into the database
//! This will probably be removed or reworked later
//!
//! Run with `cargo run --bin seed`.
//!
//! - `seeds/majors.sql` and `seeds/terms.sql` are always loaded.
//! - Members come from `sensitive/current_members.sql` when that file exists
//!   (it is gitignored), otherwise from the fictional `seeds/dev_members.sql`.

use std::io::{BufRead, BufReader};
use std::path::Path;

use diesel::connection::SimpleConnection;
use diesel::prelude::*;

const MAJORS: &str = include_str!("../../seeds/majors.sql");
const TERMS: &str = include_str!("../../seeds/terms.sql");
const DEV_MEMBERS: &str = include_str!("../../seeds/dev_members.sql");

/// Runs each line of `path` as its own statement, so one bad row doesn't stop
/// the rest. Rows the database rejects are reported and skipped.
fn load_line_by_line(
	connection: &mut PgConnection,
	path: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
	let reader = BufReader::new(std::fs::File::open(path)?);
	let mut created = 0;

	for line in reader.lines() {
		let line = line?;
		if line.trim().is_empty() {
			continue;
		}
		match connection.batch_execute(&line) {
			Ok(()) => created += 1,
			Err(e) => eprintln!("could not run `{line}`: {e}"),
		}
	}
	println!("created {created} entries from {}", path.display());
	Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
	dotenvy::dotenv().ok();
	let db_url = std::env::var("DATABASE_URL")
		.expect("Environment variables not configured!");
	let mut connection = PgConnection::establish(&db_url)?;

	connection.batch_execute(MAJORS)?;
	println!("loaded seeds/majors.sql");

	connection.batch_execute(TERMS)?;
	println!("loaded seeds/terms.sql");

	let real_members = Path::new(env!("CARGO_MANIFEST_DIR"))
		.join("sensitive/current_members.sql");
	if real_members.exists() {
		load_line_by_line(&mut connection, &real_members)?;
	} else {
		connection.batch_execute(DEV_MEMBERS)?;
		println!(
			"{} not found; loaded seeds/dev_members.sql",
			real_members.display()
		);
	}

	let p = Path::new(env!("CARGO_MANIFEST_DIR"))
		.join("sensitive/current_payments.sql");
	if p.exists() {
		load_line_by_line(&mut connection, &p)?;
	}

	Ok(())
}
