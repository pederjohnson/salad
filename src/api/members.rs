//! The officer-only member list. Every route here takes `RequireOfficer`,
//! which is what makes it officer-only (see extract.rs).
//!
//! - `GET /api/members` lists every member with whether they paid dues for the
//!   current term.
//! - `GET /api/members/{net_id}` has everything about one member.
//! - `PATCH /api/members/{net_id}` changes some of a member's details. Roles
//!   aren't changed here; they live in the staff table.

use axum::{Json, Router, extract::Path, http::StatusCode, routing::get};
use serde::Deserialize;

use super::{AppError, AppState, Db, RequireOfficer, auth::normalize_net_id};
use crate::domain::members::{
	self, MemberChanges, MemberDetail, MemberSummary,
};

pub fn router() -> Router<AppState> {
	Router::new()
		.route("/", get(list))
		.route("/{net_id}", get(detail).patch(update))
}

async fn list(
	_: RequireOfficer,
	Db(mut conn): Db,
) -> Result<Json<Vec<MemberSummary>>, AppError> {
	Ok(Json(members::list_members(&mut conn).await?))
}

async fn detail(
	_: RequireOfficer,
	Db(mut conn): Db,
	Path(net_id): Path<String>,
) -> Result<Json<MemberDetail>, AppError> {
	let net_id = normalize_net_id(&net_id)?;
	members::member_detail(&mut conn, &net_id)
		.await?
		.map(Json)
		.ok_or(AppError::NotFound("No member with that NetID"))
}

/// The body of `PATCH /api/members/{net_id}`. Send only the fields that
/// changed; the NetID in the path says whose. Unknown fields are rejected, so
/// a typo can't be silently ignored.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct UpdateRequest {
	/// Their new NetID.
	net_id: Option<String>,
	first_name: Option<String>,
	last_name: Option<String>,
	/// An empty string clears it.
	major: Option<String>,
	/// An empty string clears it.
	card_id: Option<String>,
}

/// Responds 204 if the changes were saved, 404 if there's no such member,
/// and 400 or 409 if a change is invalid (in which case nothing is saved).
async fn update(
	_: RequireOfficer,
	Db(mut conn): Db,
	Path(net_id): Path<String>,
	Json(body): Json<UpdateRequest>,
) -> Result<StatusCode, AppError> {
	let net_id = normalize_net_id(&net_id)?;

	let changes = MemberChanges {
		net_id: body.net_id.as_deref().map(normalize_net_id).transpose()?,
		first_name: required(body.first_name, "First name can't be empty")?,
		last_name: required(body.last_name, "Last name can't be empty")?,
		major: clearable(body.major),
		card_id: clearable(body.card_id),
	};

	if changes.is_empty() {
		return Err(AppError::BadRequest("Nothing to change"));
	}
	if members::update_member(&mut conn, &net_id, changes).await? {
		Ok(StatusCode::NO_CONTENT)
	} else {
		Err(AppError::NotFound("No member with that NetID"))
	}
}

/// For a column that can't be empty: trims it, and rejects an empty string.
fn required(
	value: Option<String>,
	empty_message: &'static str,
) -> Result<Option<String>, AppError> {
	match value.map(|v| v.trim().to_string()) {
		Some(v) if v.is_empty() => Err(AppError::BadRequest(empty_message)),
		v => Ok(v),
	}
}

/// For a nullable column edited in a text box: trims it, and treats an empty
/// string as clearing it (`Some(None)`).
fn clearable(value: Option<String>) -> Option<Option<String>> {
	value.map(|v| {
		let v = v.trim();
		(!v.is_empty()).then(|| v.to_string())
	})
}
