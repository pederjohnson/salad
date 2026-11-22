use chrono::NaiveDate;
use diesel::prelude::*;

use crate::schema::terms;

#[derive(Queryable, Selectable, Insertable, Identifiable, Debug)]
#[diesel(table_name = crate::schema::terms)]
#[diesel(primary_key(semester))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Term {
	semester: String,
	normal_fee_cents: i32,
	effective_date: NaiveDate,
	late_fee_cents: Option<i32>,
	due_date: Option<NaiveDate>,
}

pub fn create_term(conn: &mut PgConnection, t: &Term) -> QueryResult<usize> {
	diesel::insert_into(terms::table).values(t).execute(conn)
}
