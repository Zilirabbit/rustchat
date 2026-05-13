-- Add 'file' to the message_type check constraint and create files table

ALTER TABLE messages
DROP CONSTRAINT messages_type_valid;

ALTER TABLE messages
ADD CONSTRAINT messages_type_valid
CHECK (message_type IN ('text', 'image', 'system', 'file'));

CREATE TABLE files (
    id BIGSERIAL PRIMARY KEY,
    session_id BIGINT NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
    sender_id BIGINT NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    file_name VARCHAR(256) NOT NULL,
    file_size BIGINT NOT NULL,
    file_type VARCHAR(128) NOT NULL DEFAULT 'application/octet-stream',
    file_hash VARCHAR(64) NOT NULL,
    storage_path TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL DEFAULT (NOW() + INTERVAL '24 hours')
);

CREATE INDEX files_expires_at_idx ON files (expires_at);
