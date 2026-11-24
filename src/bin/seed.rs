//! Loads test data into the database
//! This will probably be removed or reworked later
//!
//! Run with `cargo run --bin seed`.
//!
//! - `seeds/majors.sql`, `seeds/terms.sql` and `seeds/projects.sql` are always
//!   loaded.
//! - If the gitignored `sensitive/` folder exists, real data is loaded from
//!   `current_members.sql`, `current_payments.sql` and `current_staff.sql`
//!   inside it, in that order. Those files are read at runtime, so the project
//!   still builds without them.
//! - Otherwise, the fictional `seeds/dev_members.sql` is loaded instead.

use std::io::{BufRead, BufReader};
use std::path::Path;

use diesel::connection::SimpleConnection;
use diesel::prelude::*;

const MAJORS: &str = include_str!("../../seeds/majors.sql");
const TERMS: &str = include_str!("../../seeds/terms.sql");
const PROJECTS: &str = include_str!("../../seeds/projects.sql");
const DEV_MEMBERS: &str = include_str!("../../seeds/dev_members.sql");

/// Loaded in this order, since payments and staff refer to members.
const SENSITIVE_FILES: [&str; 3] = [
	"current_members.sql",
	"current_payments.sql",
	"current_staff.sql",
];

/// Runs each line of `path` as its own statement, so one bad row doesn't stop
/// the rest. Rows the database rejects are reported and skipped. Blank lines
/// and lines that are only a comment are ignored.
fn load_line_by_line(
	connection: &mut PgConnection,
	path: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
	let reader = BufReader::new(std::fs::File::open(path)?);
	let mut created = 0;
	let mut failed = 0;

	for line in reader.lines() {
		let line = line?;
		let trimmed = line.trim();
		if trimmed.is_empty() || trimmed.starts_with("--") {
			continue;
		}
		match connection.batch_execute(trimmed) {
			Ok(()) => created += 1,
			Err(e) => {
				failed += 1;
				eprintln!("could not run `{trimmed}`: {e}");
			}
		}
	}
	println!(
		"created {created} entries from {} ({failed} failed)",
		path.display()
	);
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

	connection.batch_execute(PROJECTS)?;
	println!("loaded seeds/projects.sql");

	let sensitive = Path::new(env!("CARGO_MANIFEST_DIR")).join("sensitive");
	if !sensitive.is_dir() {
		connection.batch_execute(DEV_MEMBERS)?;
		println!(
			"{} not found; loaded seeds/dev_members.sql",
			sensitive.display()
		);
		return Ok(());
	}

	for name in SENSITIVE_FILES {
		let path = sensitive.join(name);
		if path.exists() {
			load_line_by_line(&mut connection, &path)?;
		} else {
			eprintln!("{} not found; skipping", path.display());
		}
	}

	Ok(())
}
