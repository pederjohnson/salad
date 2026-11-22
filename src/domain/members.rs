use crate::schema::members;
use chrono::{DateTime, Utc};
use diesel::prelude::*;

#[derive(Queryable, Selectable, Identifiable, Debug)]
#[diesel(table_name = crate::schema::members)]
#[diesel(primary_key(net_id))]
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
/// Invalid fields may result in an error, so the row won't be created and the error must be communicated to the user.
pub fn create_member(
	conn: &mut PgConnection,
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
	diesel::insert_into(members::table).values(m).execute(conn)
}

/// `new_major` must match a `majors.name`, or the update fails.
/// Pass `None` to clear it.
pub fn set_member_major(
	conn: &mut PgConnection,
	target_net_id: &str,
	new_major: Option<&str>,
) -> QueryResult<usize> {
	diesel::update(members::table.filter(members::net_id.eq(target_net_id)))
		.set(members::major.eq(new_major))
		.execute(conn)
}

/// Safe way to add Discord
pub fn set_member_discord(
	conn: &mut PgConnection,
	target_net_id: &str,
	new_discord: &str,
) -> QueryResult<usize> {
	diesel::update(members::table.filter(members::net_id.eq(target_net_id)))
		.set(members::discord_id.eq(new_discord))
		.execute(conn)
}

pub fn set_member_card(
	conn: &mut PgConnection,
	target_net_id: &str,
	new_card: &str,
) -> QueryResult<usize> {
	diesel::update(members::table.filter(members::net_id.eq(target_net_id)))
		.set(members::card_id.eq(new_card))
		.execute(conn)
}
