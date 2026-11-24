-- Lookup table so every member's major is spelled the same way.
-- Renaming a major cascades to members; a major in use cannot be deleted.
-- `name` is abbreviation
CREATE TABLE majors (
	name          TEXT  PRIMARY KEY,
	description   TEXT  NOT NULL UNIQUE
);

-- Required inputs: first/last name & `net_id`
-- `major` is nullable for members who are undeclared or haven't told us.
--
-- Entries are created by staff or by self-creating (verify with NetID)
--
-- TO DO: should `discord_id` and `card_id` be changed to integers?
CREATE TABLE members (
	net_id      VARCHAR(9)   PRIMARY KEY CHECK (net_id ~ '^[a-z]{3}[0-9]{6}$'),
	first_name  TEXT         NOT NULL,
	last_name   TEXT         NOT NULL,
	date_joined TIMESTAMPTZ  NOT NULL DEFAULT now(),
	discord_id  VARCHAR(30)  UNIQUE CHECK (discord_id ~ '^[0-9]+$'), -- from discord bot author id
	card_id     VARCHAR(30)  UNIQUE CHECK (card_id ~ '^[0-9]+$'),
	major       TEXT         REFERENCES majors (name) ON UPDATE CASCADE ON DELETE RESTRICT
);

CREATE INDEX members_major_idx ON members (major);

-- Effective date is unique so that "the latest term" is never ambiguous
CREATE TABLE terms (
	semester          TEXT     PRIMARY KEY CHECK (semester ~ '^(Fall|Spring) 2\d{3}$'),
	normal_fee_cents  INTEGER  NOT NULL,
	effective_date    DATE     NOT NULL UNIQUE,
	late_fee_cents    INTEGER,
	due_date          DATE,
	CONSTRAINT non_negative_price CHECK (normal_fee_cents >= 0),
	CONSTRAINT valid_late_fee CHECK (late_fee_cents >= normal_fee_cents),
	CONSTRAINT effective_before_due CHECK (effective_date < due_date),
	CONSTRAINT due_date_needs_late_fee CHECK ((late_fee_cents IS NULL) = (due_date IS NULL))
);

-- Paypal, Venmo, and Cashapp support are depricated.
-- Square is currently being sunset for Stripe
CREATE TYPE payment_method AS ENUM ('cash', 'stripe', 'square', 'paypal', 'venmo', 'cashapp', 'other');

-- Which term a payment counts toward is determined by time_purchased alone
-- (see has_paid_term in the dues_logic migration), so there is no term column.
--
-- If someone wants to pay for a future semester, we can do something in the
-- backend to change time_purchased manually. If it's a card, the metadata shows
-- the actual transaction time. If it's cash, the metadata notes the change.
--
-- Payments made with a typo'd NetID can be fixed by staff but should be mediated
-- against by authentication in the future.
--
-- TO DO: Should we constrain `amount_paid_cents` >= the amount due at that time?
-- TO DO: How do we handle refunds?
-- TO DO: A payment handler webhook might send the same event multiple times. Do
-- we need to pull something like make `metadata->>'stripe_id'` unique?
CREATE TABLE term_payments (
	id                 UUID            PRIMARY KEY DEFAULT gen_random_uuid(),
	net_id             VARCHAR(9)      NOT NULL REFERENCES members (net_id) ON UPDATE CASCADE ON DELETE RESTRICT,
	time_purchased     TIMESTAMPTZ     NOT NULL DEFAULT now(),
	amount_paid_cents  INTEGER         NOT NULL,
	method             payment_method  NOT NULL,
	metadata           JSONB           NOT NULL DEFAULT '{}'
);

-- Renaming a project cascades to staff assignments; a project with staff
-- cannot be deleted (mark it inactive instead).
CREATE TABLE projects (
	name         TEXT     PRIMARY KEY,
	description  TEXT,
	active       BOOLEAN  NOT NULL DEFAULT true
);

-- An officer or lead is a member, so net_id is both the primary key and a
-- reference to members (at most one staff row per member).
--
-- Officers have no project; leads have exactly one. Because there is one row
-- per member, nobody can be an officer and a lead at once (see our
-- constitution).
--
-- `officer` is derived from `project` rather than set by hand, so the two can
-- never disagree. To make someone an officer, insert a row with project NULL;
-- to make them a lead, insert a row with their project.
CREATE TABLE staff (
	net_id   VARCHAR(9)  PRIMARY KEY REFERENCES members (net_id) ON UPDATE CASCADE ON DELETE RESTRICT,
	project  TEXT        REFERENCES projects (name) ON UPDATE CASCADE ON DELETE RESTRICT,
	officer  BOOLEAN     NOT NULL GENERATED ALWAYS AS (project IS NULL) STORED
);

CREATE INDEX staff_project_idx ON staff (project);

-- Create events and attendances later
