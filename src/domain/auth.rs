//! Sign-in by emailed code, and the sessions it creates.
//! See migrations/0003_auth for the flow and the tables.

use chrono::{DateTime, Utc};
use diesel::deserialize;
use diesel::dsl::now;
use diesel::pg::Pg;
use diesel::prelude::*;
use diesel::sql_types::{Bool, Nullable, Text};
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::schema::{members, sessions, staff};

fn sha256(input: &str) -> Vec<u8> {
	Sha256::digest(input.as_bytes()).to_vec()
}

mod sql {
	use diesel::expression::functions::declare_sql_function;
	use diesel::sql_types::{Bool, Bytea, Varchar};

	// Defined in migrations/0003_auth. Arguments are passed by name, so these
	// parameter names must match the SQL function's.
	#[declare_sql_function(named_parameters = true)]
	extern "SQL" {
		fn issue_login_code(
			member_net_id: Varchar,
			new_code_hash: Bytea,
		) -> Bool;

		fn redeem_login_code(
			member_net_id: Varchar,
			guess_hash: Bytea,
			accept_any: Bool,
		) -> Bool;
	}
}

/// Generates a 6-digit code for `net_id` and returns it so the caller can
/// email it. Returns `None` if `net_id` isn't a member or a code was issued
/// within the last 60 seconds; send nothing in that case. The rules live in
/// the `issue_login_code` SQL function, which stores only the code's hash.
pub async fn issue_login_code(
	conn: &mut AsyncPgConnection,
	net_id: &str,
) -> QueryResult<Option<String>> {
	let code = format!("{:06}", rand::random_range(0..1_000_000u32));
	let issued: bool =
		diesel::select(sql::issue_login_code(net_id, sha256(&code)))
			.get_result(conn)
			.await?;
	Ok(issued.then_some(code))
}

/// Checks `code` against the pending code for `net_id`. Returns `true` and
/// uses the code up on a match; a wrong guess counts toward the 5 allowed.
/// The rules live in the `redeem_login_code` SQL function.
///
/// With `accept_any`, any code counts as a match, but a code must still have
/// been requested. This exists only until emails are actually sent.
pub async fn redeem_login_code(
	conn: &mut AsyncPgConnection,
	net_id: &str,
	code: &str,
	accept_any: bool,
) -> QueryResult<bool> {
	diesel::select(sql::redeem_login_code(net_id, sha256(code), accept_any))
		.get_result(conn)
		.await
}

/// Starts a session for `net_id` and returns its token (to give to the
/// client, never stored) and when it expires. The database sets the expiry
/// (see the sessions table in migrations/0003_auth). Also clears out this
/// member's expired sessions.
pub async fn create_session(
	conn: &mut AsyncPgConnection,
	net_id: &str,
) -> QueryResult<(String, DateTime<Utc>)> {
	diesel::delete(
		sessions::table
			.filter(sessions::net_id.eq(net_id))
			.filter(sessions::expires_at.le(now)),
	)
	.execute(conn)
	.await?;

	let token = hex::encode(rand::random::<[u8; 32]>());
	let expires_at = diesel::insert_into(sessions::table)
		.values((
			sessions::token_hash.eq(sha256(&token)),
			sessions::net_id.eq(net_id),
		))
		.returning(sessions::expires_at)
		.get_result(conn)
		.await?;
	Ok((token, expires_at))
}

/// What a member can do, from their staff row (if any).
///
/// In JSON this is `"role": "member" | "lead" | "officer"`, plus
/// `"project": ...` for a lead. Use `#[serde(flatten)]` to put those fields
/// directly in the containing object.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "role", rename_all = "lowercase")]
pub enum Role {
	Member,
	Lead { project: String },
	Officer,
}

impl Role {
	/// The columns to select to load a `Role`, for example
	/// `.select((members::net_id, Role::columns()))` loads `(String, Role)`.
	/// The query must left-join `staff`, so that both columns are NULL for
	/// someone with no staff row.
	pub fn columns() -> (
		diesel::dsl::Nullable<staff::officer>,
		diesel::dsl::Nullable<staff::project>,
	) {
		(staff::officer.nullable(), staff::project.nullable())
	}
}

/// Lets Diesel build a `Role` from `Role::columns()`.
impl Queryable<(Nullable<Bool>, Nullable<Text>), Pg> for Role {
	type Row = (Option<bool>, Option<String>);

	fn build((officer, project): Self::Row) -> deserialize::Result<Self> {
		Ok(match (officer, project) {
			(Some(true), _) => Role::Officer,
			(Some(false), Some(project)) => Role::Lead { project },
			// officer is generated as `project IS NULL`, so a staff row
			// with officer = false always has a project.
			_ => Role::Member,
		})
	}
}

#[derive(Queryable, Debug, Clone)]
pub struct SessionMember {
	pub net_id: String,
	pub first_name: String,
	pub last_name: String,
	pub role: Role,
}

/// The member a session token belongs to, or `None` if the token is unknown
/// or expired.
pub async fn session_member(
	conn: &mut AsyncPgConnection,
	token: &str,
) -> QueryResult<Option<SessionMember>> {
	sessions::table
		.inner_join(members::table.left_join(staff::table))
		.filter(sessions::token_hash.eq(sha256(token)))
		.filter(sessions::expires_at.gt(now))
		.select((
			members::net_id,
			members::first_name,
			members::last_name,
			Role::columns(),
		))
		.first(conn)
		.await
		.optional()
}

/// Ends the session for `token`. Does nothing if it doesn't exist.
pub async fn delete_session(
	conn: &mut AsyncPgConnection,
	token: &str,
) -> QueryResult<()> {
	diesel::delete(sessions::table.find(sha256(token)))
		.execute(conn)
		.await?;
	Ok(())
}
