------------------------------------------------------------------------------
-- Dues logic
--
-- Term dates (effective_date, due_date) are calendar days in the club's local
-- time zone, while time_purchased is an absolute instant. club_date() converts
-- an instant to the club's calendar day so the two can be compared regardless
-- of the server's TimeZone setting (Azure defaults to UTC).
-- This can prevent scenarios like paying dues at 10pm being counted as late.
------------------------------------------------------------------------------

CREATE FUNCTION club_date(at TIMESTAMPTZ) RETURNS DATE
LANGUAGE sql IMMUTABLE STRICT PARALLEL SAFE
RETURN (at AT TIME ZONE 'America/Chicago')::DATE;

-- The term in effect at a given moment: the one with the latest effective
-- date that has already started. Terms defined ahead of time (effective date
-- in the future) are ignored until they begin. Returns no row if no term has
-- started yet.
CREATE FUNCTION current_term(at TIMESTAMPTZ DEFAULT now()) RETURNS SETOF terms
LANGUAGE sql STABLE PARALLEL SAFE
BEGIN ATOMIC
	SELECT *
	FROM terms
	WHERE effective_date <= club_date(at)
	ORDER BY effective_date DESC
	LIMIT 1;
END;

-- The price to charge at a given moment: the current term's late fee if we
-- are past its due date, otherwise its normal fee. NULL if no term has
-- started yet. Paying on the due date itself is not late.
CREATE FUNCTION current_dues_price(at TIMESTAMPTZ DEFAULT now()) RETURNS INTEGER
LANGUAGE sql STABLE PARALLEL SAFE
BEGIN ATOMIC
	SELECT CASE
		WHEN t.due_date IS NOT NULL AND club_date(at) > t.due_date THEN t.late_fee_cents
		ELSE t.normal_fee_cents
	END
	FROM current_term(at) AS t;
END;

-- Whether a member paid for a specific term: some payment of theirs happened
-- on or after that term's effective date and before the next term's effective
-- date (or at any later time, if no later term exists yet). False if the term
-- doesn't exist.
CREATE FUNCTION has_paid_term(member_net_id VARCHAR, semester_name TEXT) RETURNS BOOLEAN
LANGUAGE sql STABLE PARALLEL SAFE
BEGIN ATOMIC
	SELECT EXISTS (
		SELECT 1
		FROM terms AS t
		JOIN term_payments AS p
			ON p.net_id = member_net_id
			AND club_date(p.time_purchased) >= t.effective_date
		WHERE t.semester = semester_name
			AND club_date(p.time_purchased) < COALESCE(
				(SELECT min(n.effective_date) FROM terms AS n WHERE n.effective_date > t.effective_date),
				'infinity'::DATE
			)
	);
END;

-- Whether a member has paid for the term currently in effect.
CREATE FUNCTION has_paid_current_term(member_net_id VARCHAR) RETURNS BOOLEAN
LANGUAGE sql STABLE PARALLEL SAFE
BEGIN ATOMIC
	SELECT COALESCE(
		(SELECT has_paid_term(member_net_id, t.semester) FROM current_term() AS t),
		FALSE
	);
END;

-- Every member and whether they've paid for the term currently in effect.
-- `term` is NULL (and `paid` false) if no term has started yet.
CREATE VIEW member_dues_status AS
SELECT
	m.net_id,
	m.first_name,
	m.last_name,
	t.semester AS term,
	COALESCE(has_paid_term(m.net_id, t.semester), FALSE) AS paid
FROM members AS m
LEFT JOIN current_term() AS t ON TRUE;

-- Supports the "did this member pay in this window" lookups in has_paid_term.
CREATE INDEX term_payments_net_id_time_idx ON term_payments (net_id, time_purchased);
