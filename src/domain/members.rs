use chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use serde::Serialize;

use crate::domain::auth::Role;
use crate::domain::payments::{
	self, TermPayment,
	sql::{has_paid_current_term, has_paid_term},
};
use crate::schema::{members, staff, terms};

/// A row of the `members` table. Select it with `Member::as_select()`, and
/// embed it in a response with `#[serde(flatten)]` to send every column.
#[derive(Queryable, Selectable, Serialize, Debug)]
#[diesel(table_name = crate::schema::members)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Member {
	net_id: String,
	first_name: String,
	last_name: String,
	date_joined: DateTime<Utc>,
	discord_id: Option<String>,
	card_id: Option<String>,
	major: Option<String>,
}

#[derive(Insertable, Debug)]
#[diesel(table_name = crate::schema::members)]
#[diesel(check_for_backend(diesel::pg::Pg))]
struct NewMember {
	net_id: String,
	first_name: String,
	last_name: String,
	major: Option<String>,
}

/// This is the gateway to creating a new member.
///
/// Invalid fields may result in an error, so the row won't be created and
/// the error must be communicated to the user.
pub async fn create_member(
	conn: &mut AsyncPgConnection,
	net_id: String,
	first_name: String,
	last_name: String,
	major: Option<String>,
) -> QueryResult<usize> {
	let m = NewMember {
		net_id,
		first_name,
		last_name,
		major,
	};
	diesel::insert_into(members::table)
		.values(m)
		.execute(conn)
		.await
}

/// `new_major` must match a `majors.name`, or the update fails.
/// Pass `None` to clear it.
pub async fn set_member_major(
	conn: &mut AsyncPgConnection,
	target_net_id: &str,
	new_major: Option<&str>,
) -> QueryResult<usize> {
	diesel::update(members::table.filter(members::net_id.eq(target_net_id)))
		.set(members::major.eq(new_major))
		.execute(conn)
		.await
}

/// Safe way to add Discord
pub async fn set_member_discord(
	conn: &mut AsyncPgConnection,
	target_net_id: &str,
	new_discord: &str,
) -> QueryResult<usize> {
	diesel::update(members::table.filter(members::net_id.eq(target_net_id)))
		.set(members::discord_id.eq(new_discord))
		.execute(conn)
		.await
}

pub async fn set_member_card(
	conn: &mut AsyncPgConnection,
	target_net_id: &str,
	new_card: &str,
) -> QueryResult<usize> {
	diesel::update(members::table.filter(members::net_id.eq(target_net_id)))
		.set(members::card_id.eq(new_card))
		.execute(conn)
		.await
}

/// One row of the members table that officers see. Loaded straight from
/// the query in `list_members`, whose columns must be in this order.
///
/// This is meant exclusively for a query joining members and staff tables
#[derive(Queryable, Serialize, Debug)]
pub struct MemberSummary {
	pub net_id: String,
	pub first_name: String,
	pub last_name: String,
	/// Whether they paid dues for the current term.
	pub paid: bool,
	#[serde(flatten)]
	pub role: Role,
}

/// Every member, sorted by last name, whether or not they're active.
pub async fn list_members(
	conn: &mut AsyncPgConnection,
) -> QueryResult<Vec<MemberSummary>> {
	members::table
		.left_join(staff::table)
		.select((
			members::net_id,
			members::first_name,
			members::last_name,
			has_paid_current_term(members::net_id),
			Role::columns(),
		))
		.order((members::last_name, members::first_name))
		.load(conn)
		.await
}

/// Everything about one member, for officers.
#[derive(Serialize, Debug)]
pub struct MemberDetail {
	/// Every column of their `members` row.
	#[serde(flatten)]
	pub member: Member,
	#[serde(flatten)]
	pub role: Role,
	/// Every term they paid dues for, newest first.
	pub terms_paid: Vec<String>,
	/// Newest first.
	pub payments: Vec<TermPayment>,
}

/// `None` if `net_id` isn't a member.
pub async fn member_detail(
	conn: &mut AsyncPgConnection,
	net_id: &str,
) -> QueryResult<Option<MemberDetail>> {
	let row: Option<(Member, Role)> = members::table
		.left_join(staff::table)
		.filter(members::net_id.eq(net_id))
		.select((Member::as_select(), Role::columns()))
		.first(conn)
		.await
		.optional()?;
	let Some((member, role)) = row else {
		return Ok(None);
	};

	let terms_paid = terms::table
		.filter(has_paid_term(net_id, terms::semester))
		.order(terms::effective_date.desc())
		.select(terms::semester)
		.load(conn)
		.await?;
	let payments = payments::payments_for(conn, net_id).await?;

	Ok(Some(MemberDetail {
		member,
		role,
		terms_paid,
		payments,
	}))
}
