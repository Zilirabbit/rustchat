-- Enforce one private session per unordered user pair.
--
-- This migration intentionally removes all existing private sessions first.
-- Cascading foreign keys clean up related private messages, members and read
-- states while leaving group sessions untouched.

UPDATE sessions
SET last_message_id = NULL,
    last_message_at = NULL
WHERE session_type = 'private';

DELETE FROM sessions
WHERE session_type = 'private';

CREATE TABLE private_session_pairs (
    session_id BIGINT PRIMARY KEY REFERENCES sessions(id) ON DELETE CASCADE,
    user_low_id BIGINT NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    user_high_id BIGINT NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT private_session_pairs_ordered CHECK (user_low_id < user_high_id),
    CONSTRAINT private_session_pairs_user_pair_uidx UNIQUE (user_low_id, user_high_id)
);

CREATE INDEX private_session_pairs_high_low_idx
ON private_session_pairs (user_high_id, user_low_id);
