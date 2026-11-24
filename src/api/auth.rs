//! Sign-in endpoints and the `AuthUser` extractor.
//!
//! 1. `POST /api/auth/code` with `{"net_id": "abc123456"}` emails a code to
//!    abc123456@utdallas.edu.
//! 2. `POST /api/auth/verify` with `{"net_id": ..., "code": "123456"}`
//!    returns `{"token": ..., "expires_at": ...}`.
//! 3. Send `Authorization: Bearer <token>` on later requests.

use axum::{
	Json, Router,
	extract::FromRequestParts,
	http::{StatusCode, header, request::Parts},
	routing::{get, post},
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use super::{AppError, AppState, Db};
use crate::domain::auth::{self, Role, SessionMember};

/// Until sign-in emails are sent, debug builds accept any 6-digit code.
/// Release builds check the code for real, so nobody can sign in to a
/// release build until sending is implemented in `send_login_code`.
const ACCEPT_ANY_CODE: bool = cfg!(debug_assertions);

pub fn router() -> Router<AppState> {
	Router::new()
		.route("/code", post(request_code))
		.route("/verify", post(verify_code))
		.route("/me", get(me))
		.route("/logout", post(logout))
}

/// Trims and lowercases a NetID, then checks it looks like abc123456.
fn normalize_net_id(raw: &str) -> Result<String, AppError> {
	let net_id = raw.trim().to_ascii_lowercase();
	let (letters, digits) = net_id.split_at_checked(3).unwrap_or(("", ""));
	let valid = letters.len() == 3
		&& letters.bytes().all(|b| b.is_ascii_lowercase())
		&& digits.len() == 6
		&& digits.bytes().all(|b| b.is_ascii_digit());
	if valid {
		Ok(net_id)
	} else {
		Err(AppError::BadRequest("NetID should look like abc123456"))
	}
}

/// TODO: email `code` to `net_id`@utdallas.edu. Never log the code itself.
fn send_login_code(net_id: &str, _code: &str) {
	tracing::info!(
		"not sending sign-in code to {net_id}@utdallas.edu: email isn't set up"
	);
}

#[derive(Deserialize)]
struct CodeRequest {
	net_id: String,
}

/// Always answers the same way for a valid NetID, member or not, so this
/// can't be used to find out who is in the club.
async fn request_code(
	Db(mut conn): Db,
	Json(body): Json<CodeRequest>,
) -> Result<(StatusCode, Json<Value>), AppError> {
	let net_id = normalize_net_id(&body.net_id)?;

	if let Some(code) = auth::issue_login_code(&mut conn, &net_id).await? {
		send_login_code(&net_id, &code);
	}

	Ok((
		StatusCode::ACCEPTED,
		Json(json!({
			"message": format!(
				"If {net_id} is a member, a code was sent to {net_id}@utdallas.edu"
			),
		})),
	))
}

#[derive(Deserialize)]
struct VerifyRequest {
	net_id: String,
	code: String,
}

#[derive(Serialize)]
struct SessionResponse {
	token: String,
	expires_at: DateTime<Utc>,
}

async fn verify_code(
	Db(mut conn): Db,
	Json(body): Json<VerifyRequest>,
) -> Result<Json<SessionResponse>, AppError> {
	let net_id = normalize_net_id(&body.net_id)?;
	let code = body.code.trim().to_string();
	if code.len() != 6 || !code.bytes().all(|b| b.is_ascii_digit()) {
		return Err(AppError::BadRequest("The code should be 6 digits"));
	}

	if !auth::redeem_login_code(&mut conn, &net_id, &code, ACCEPT_ANY_CODE)
		.await?
	{
		return Err(AppError::Unauthorized);
	}
	let (token, expires_at) = auth::create_session(&mut conn, &net_id).await?;
	Ok(Json(SessionResponse { token, expires_at }))
}

/// The signed-in member. Add it as a handler argument to require sign-in;
/// requests without a valid `Authorization: Bearer <token>` get a 401.
pub struct AuthUser {
	pub member: SessionMember,
	token: String,
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

#[derive(Serialize)]
struct MeResponse {
	net_id: String,
	first_name: String,
	last_name: String,
	/// "member", "lead", or "officer"
	role: &'static str,
	/// The project a lead runs; null for everyone else.
	project: Option<String>,
}

async fn me(user: AuthUser) -> Json<MeResponse> {
	let m = user.member;
	let (role, project) = match m.role {
		Role::Member => ("member", None),
		Role::Lead { project } => ("lead", Some(project)),
		Role::Officer => ("officer", None),
	};
	Json(MeResponse {
		net_id: m.net_id,
		first_name: m.first_name,
		last_name: m.last_name,
		role,
		project,
	})
}

async fn logout(
	user: AuthUser,
	Db(mut conn): Db,
) -> Result<StatusCode, AppError> {
	auth::delete_session(&mut conn, &user.token).await?;
	Ok(StatusCode::NO_CONTENT)
}
