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

/// Changes to one member's `members` row, turned into a single `UPDATE` by
/// Diesel's `AsChangeset`. Each field is a column, and the outer `Option` says
/// whether to touch it: `None` leaves the column out of the `UPDATE`.
///
/// Columns that can be NULL (`major`, `card_id`) need a second `Option` for the
/// value itself, because an edit can do three different things to them:
///
/// | Value                 | SQL                     | Meaning           |
/// |-----------------------|-------------------------|-------------------|
/// | `None`                | (column left out)       | keep it as it is  |
/// | `Some(None)`          | `SET major = NULL`      | clear it          |
/// | `Some(Some("CS"))`    | `SET major = 'CS'`      | change it         |
///
/// A plain `Option<String>` can only say two of these. Diesel's alternative,
/// `#[diesel(treat_none_as_null = true)]`, makes `None` mean "clear", so every
/// update would wipe the major unless the caller re-sent it.
///
/// Columns that can't be NULL (`net_id`, the names) only need "keep" or
/// "change", so they have a single `Option`.
#[derive(AsChangeset, Default, Debug)]
#[diesel(table_name = crate::schema::members)]
pub struct MemberChanges {
	pub net_id: Option<String>,
	pub first_name: Option<String>,
	pub last_name: Option<String>,
	/// Must match a `majors.name`. See above for why this is a double `Option`.
	pub major: Option<Option<String>>,
	/// See above for why this is a double `Option`.
	pub card_id: Option<Option<String>>,
}

impl MemberChanges {
	pub fn is_empty(&self) -> bool {
		// Listing every field means adding one won't compile until it's
		// handled here too.
		let MemberChanges {
			net_id,
			first_name,
			last_name,
			major,
			card_id,
		} = self;
		net_id.is_none()
			&& first_name.is_none()
			&& last_name.is_none()
			&& major.is_none()
			&& card_id.is_none()
	}
}

/// Updates `net_id`'s row with `changes` in a single `UPDATE`. Never creates
/// a member: returns `false` if `net_id` isn't one. `changes` must not be
/// empty, or Diesel refuses to build the query.
///
/// A new NetID carries over to their staff row, payments and sessions
/// (`ON UPDATE CASCADE`).
pub async fn update_member(
	conn: &mut AsyncPgConnection,
	net_id: &str,
	changes: MemberChanges,
) -> QueryResult<bool> {
	let updated = diesel::update(members::table.find(net_id))
		.set(&changes)
		.execute(conn)
		.await?;
	Ok(updated == 1)
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
