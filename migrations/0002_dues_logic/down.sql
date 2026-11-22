-- This file should undo anything in `up.sql`
DROP INDEX term_payments_net_id_time_idx;
DROP VIEW member_dues_status;
DROP FUNCTION has_paid_current_term(VARCHAR);
DROP FUNCTION has_paid_term(VARCHAR, TEXT);
DROP FUNCTION current_dues_price(TIMESTAMPTZ);
DROP FUNCTION current_term(TIMESTAMPTZ);
DROP FUNCTION club_date(TIMESTAMPTZ);
