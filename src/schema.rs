// @generated automatically by Diesel CLI.

pub mod sql_types {
	#[derive(
		Clone, diesel::query_builder::QueryId, diesel::sql_types::SqlType,
	)]
	#[diesel(postgres_type(name = "payment_method"))]
	#[diesel(enum_type)]
	pub struct PaymentMethod;
}

pub mod rust_types {
	#[derive(Clone, Copy, Debug, Eq, PartialEq, diesel::types::Enum)]
	#[diesel(sql_type = super::sql_types::PaymentMethod)]
	pub enum PaymentMethod {
		#[diesel(rename = "cash")]
		Cash,
		#[diesel(rename = "stripe")]
		Stripe,
		#[diesel(rename = "square")]
		Square,
		#[diesel(rename = "paypal")]
		Paypal,
		#[diesel(rename = "venmo")]
		Venmo,
		#[diesel(rename = "cashapp")]
		Cashapp,
		#[diesel(rename = "other")]
		Other,
	}
}

diesel::table! {
	login_codes (net_id) {
		#[max_length = 9]
		net_id -> Varchar,
		code_hash -> Bytea,
		created_at -> Timestamptz,
		expires_at -> Timestamptz,
		attempts -> Int4,
	}
}

diesel::table! {
	majors (name) {
		name -> Text,
		description -> Text,
	}
}

diesel::table! {
	members (net_id) {
		#[max_length = 9]
		net_id -> Varchar,
		first_name -> Text,
		last_name -> Text,
		date_joined -> Timestamptz,
		#[max_length = 30]
		discord_id -> Nullable<Varchar>,
		#[max_length = 30]
		card_id -> Nullable<Varchar>,
		major -> Nullable<Text>,
	}
}

diesel::table! {
	projects (name) {
		name -> Text,
		description -> Nullable<Text>,
		active -> Bool,
	}
}

diesel::table! {
	sessions (token_hash) {
		token_hash -> Bytea,
		#[max_length = 9]
		net_id -> Varchar,
		created_at -> Timestamptz,
		expires_at -> Timestamptz,
	}
}

diesel::table! {
	staff (net_id) {
		#[max_length = 9]
		net_id -> Varchar,
		project -> Nullable<Text>,
		officer -> Bool,
	}
}

diesel::table! {
	use diesel::sql_types::*;
	use super::sql_types::PaymentMethod;

	term_payments (id) {
		id -> Uuid,
		#[max_length = 9]
		net_id -> Varchar,
		time_purchased -> Timestamptz,
		amount_paid_cents -> Int4,
		method -> PaymentMethod,
		metadata -> Jsonb,
	}
}

diesel::table! {
	terms (semester) {
		semester -> Text,
		normal_fee_cents -> Int4,
		effective_date -> Date,
		late_fee_cents -> Nullable<Int4>,
		due_date -> Nullable<Date>,
	}
}

diesel::joinable!(login_codes -> members (net_id));
diesel::joinable!(members -> majors (major));
diesel::joinable!(sessions -> members (net_id));
diesel::joinable!(staff -> members (net_id));
diesel::joinable!(staff -> projects (project));
diesel::joinable!(term_payments -> members (net_id));

diesel::allow_tables_to_appear_in_same_query!(
	login_codes,
	majors,
	members,
	projects,
	sessions,
	staff,
	term_payments,
	terms,
);
