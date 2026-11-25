//! Extractors: types a handler takes as arguments to get something from the
//! request, or to require something before the handler runs.
//!
//! - `Db`: a database connection.
//! - `AuthUser`: the signed-in member (401 if not signed in).
//! - `RequireOfficer`: the signed-in member, who must be an officer (403
//!   otherwise).

use axum::{
	extract::FromRequestParts,
	http::{header, request::Parts},
};
use diesel_async::{AsyncPgConnection, pooled_connection::deadpool::Object};

use super::{AppError, AppState};
use crate::domain::auth::{self, Role, SessionMember};

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

/// The signed-in member. Add it as a handler argument to require sign-in;
/// requests without a valid `Authorization: Bearer <token>` get a 401.
pub struct AuthUser {
	pub member: SessionMember,
	/// The raw session token, so `logout` can end this session.
	pub(super) token: String,
}

impl FromRequestParts<AppState> for AuthUser {
	type Rejection = AppError;

	async fn from_request_parts(
		parts: &mut Parts,
		state: &AppState,
	) -> Result<Self, Self::Rejection> {
		// TO DO: can this be made simpler?
		let token = parts
			.headers
			.get(header::AUTHORIZATION)
			.and_then(|v| v.to_str().ok())
			.and_then(|v| v.strip_prefix("Bearer "))
			.map(|t| t.trim().to_string())
			.ok_or(AppError::Unauthorized)?;

		let Db(mut conn) = Db::from_request_parts(parts, state).await?;
		let member = auth::session_member(&mut conn, &token)
			.await?
			.ok_or(AppError::Unauthorized)?;
		Ok(AuthUser { member, token })
	}
}

/// Requires the signed-in member to be an officer (`Role::Officer`) and gives
/// you their details. Add it as a handler argument to make a route
/// officer-only: others get a 403, or a 401 if not signed in.
///
/// The check happens while axum builds the handler's arguments, so the handler
/// body never runs for anyone else. Taking `_: RequireOfficer` is the whole
/// check; there's no separate call inside the handler that could be forgotten.
pub struct RequireOfficer(pub SessionMember);

impl FromRequestParts<AppState> for RequireOfficer {
	type Rejection = AppError;

	async fn from_request_parts(
		parts: &mut Parts,
		state: &AppState,
	) -> Result<Self, Self::Rejection> {
		let user = AuthUser::from_request_parts(parts, state).await?;
		match user.member.role {
			Role::Officer => Ok(RequireOfficer(user.member)),
			_ => Err(AppError::Forbidden),
		}
	}
}
