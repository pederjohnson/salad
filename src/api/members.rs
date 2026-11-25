//! The officer-only member list. Every route here takes `RequireOfficer`,
//! which is what makes it officer-only (see extract.rs).
//!
//! - `GET /api/members` lists every member with whether they paid dues for the
//!   current term.
//! - `GET /api/members/{net_id}` has everything about one member.

use axum::{Json, Router, extract::Path, routing::get};

use super::{AppError, AppState, Db, RequireOfficer, auth::normalize_net_id};
use crate::domain::members::{self, MemberDetail, MemberSummary};

pub fn router() -> Router<AppState> {
	Router::new()
		.route("/", get(list))
		.route("/{net_id}", get(detail))
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
