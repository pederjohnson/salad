------------------------------------------------------------------------------
-- Sign-in by emailed code
--
-- 1. A member enters their NetID. We create a login_codes row and email a
--    6-digit code to <net_id>@utdallas.edu.
-- 2. They enter the code. If it matches an unexpired row, the row is deleted
--    and a session is created; the client gets a random token.
--
-- Codes and tokens are stored only as SHA-256 hashes, so a leaked database
-- can't be used to sign in.
------------------------------------------------------------------------------

-- At most one pending code per member: asking again replaces the old code.
-- `attempts` counts wrong guesses so a code can't be brute-forced.
CREATE TABLE login_codes (
	net_id      VARCHAR(9)   PRIMARY KEY REFERENCES members (net_id) ON UPDATE CASCADE ON DELETE CASCADE,
	code_hash   BYTEA        NOT NULL,
	created_at  TIMESTAMPTZ  NOT NULL DEFAULT now(),
	expires_at  TIMESTAMPTZ  NOT NULL,
	attempts    INTEGER      NOT NULL DEFAULT 0
);

-- Stores a new sign-in code for a member and returns whether it did. The
-- app emails the code only when this returns true. Returns false, changing
-- nothing, when:
--   * the NetID isn't a member, or
--   * their last code was issued under 60 seconds ago. Each code allows 5
--     guesses, so without this wait someone could keep requesting codes to
--     guess as often as they like. The pending code stays valid.
--
-- The app generates the code and passes only its hash: Postgres's random()
-- isn't secure enough for codes, and its secure generator needs the pgcrypto
-- extension.
CREATE FUNCTION issue_login_code(member_net_id VARCHAR, new_code_hash BYTEA) RETURNS BOOLEAN
LANGUAGE plpgsql VOLATILE AS $$
BEGIN
	-- Clear out every expired code (Postgres can't delete rows on a timer),
	-- and this member's code if it's past the 60-second wait.
	DELETE FROM login_codes
	WHERE expires_at <= now()
		OR (net_id = member_net_id AND created_at <= now() - INTERVAL '60 seconds');

	-- Selecting from members inserts nothing for a non-member. If their code
	-- survived the delete, it's within the wait, so the insert is skipped.
	-- created_at and attempts take their defaults.
	INSERT INTO login_codes (net_id, code_hash, expires_at)
	SELECT net_id, new_code_hash, now() + INTERVAL '10 minutes'
	FROM members
	WHERE net_id = member_net_id
	ON CONFLICT (net_id) DO NOTHING;

	RETURN FOUND;
END;
$$;

-- Checks a guess against a member's pending code. A correct guess on an
-- unexpired code with fewer than 5 wrong guesses uses the code up and returns
-- true. Anything else counts as a wrong guess and returns false.
--
-- accept_any skips the comparison (a code must still have been requested).
-- It exists only until the app actually emails codes; never pass true then.
CREATE FUNCTION redeem_login_code(member_net_id VARCHAR, guess_hash BYTEA, accept_any BOOLEAN) RETURNS BOOLEAN
LANGUAGE plpgsql VOLATILE AS $$
BEGIN
	DELETE FROM login_codes
	WHERE net_id = member_net_id
		AND expires_at > now()
		AND attempts < 5
		AND (accept_any OR code_hash = guess_hash);
	IF FOUND THEN
		RETURN TRUE;
	END IF;

	-- Expired or used-up codes stay until issue_login_code replaces them;
	-- the checks above already make them useless.
	UPDATE login_codes SET attempts = attempts + 1 WHERE net_id = member_net_id;
	RETURN FALSE;
END;
$$;

-- A member may be signed in on several devices, so several sessions each.
-- Signing out deletes the row. A session lasts 14 days, after which the
-- member must sign in again.
CREATE TABLE sessions (
	token_hash  BYTEA        PRIMARY KEY,
	net_id      VARCHAR(9)   NOT NULL REFERENCES members (net_id) ON UPDATE CASCADE ON DELETE CASCADE,
	created_at  TIMESTAMPTZ  NOT NULL DEFAULT now(),
	expires_at  TIMESTAMPTZ  NOT NULL DEFAULT now() + INTERVAL '14 days'
);

CREATE INDEX sessions_net_id_idx ON sessions (net_id);
