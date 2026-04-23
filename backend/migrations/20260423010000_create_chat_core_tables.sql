-- Phase 1 / 3.4 chat core schema migration

CREATE OR REPLACE FUNCTION set_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TABLE users (
    id BIGSERIAL PRIMARY KEY,
    username VARCHAR(32) NOT NULL,
    password_hash TEXT NOT NULL,
    avatar_url TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT users_username_trimmed CHECK (username = BTRIM(username)),
    CONSTRAINT users_username_length CHECK (CHAR_LENGTH(username) BETWEEN 3 AND 32)
);

CREATE UNIQUE INDEX users_username_lower_uidx ON users (LOWER(username));

CREATE TRIGGER trg_users_set_updated_at
BEFORE UPDATE ON users
FOR EACH ROW
EXECUTE FUNCTION set_updated_at();

CREATE TABLE sessions (
    id BIGSERIAL PRIMARY KEY,
    session_type VARCHAR(16) NOT NULL,
    name VARCHAR(100),
    created_by BIGINT NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    last_message_id BIGINT,
    last_message_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT sessions_type_valid CHECK (session_type IN ('private', 'group')),
    CONSTRAINT sessions_name_rule CHECK (
        (session_type = 'private' AND name IS NULL)
        OR (
            session_type = 'group'
            AND name IS NOT NULL
            AND name = BTRIM(name)
            AND CHAR_LENGTH(name) BETWEEN 1 AND 100
        )
    ),
    CONSTRAINT sessions_last_message_pair CHECK (
        (last_message_id IS NULL AND last_message_at IS NULL)
        OR (last_message_id IS NOT NULL AND last_message_at IS NOT NULL)
    )
);

CREATE INDEX sessions_created_by_idx ON sessions (created_by);
CREATE INDEX sessions_last_message_at_idx ON sessions (last_message_at DESC NULLS LAST);

CREATE TRIGGER trg_sessions_set_updated_at
BEFORE UPDATE ON sessions
FOR EACH ROW
EXECUTE FUNCTION set_updated_at();

CREATE TABLE session_members (
    id BIGSERIAL PRIMARY KEY,
    session_id BIGINT NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
    user_id BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role VARCHAR(16) NOT NULL DEFAULT 'member',
    joined_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT session_members_role_valid CHECK (role IN ('owner', 'member')),
    CONSTRAINT session_members_unique UNIQUE (session_id, user_id)
);

CREATE INDEX session_members_user_id_idx ON session_members (user_id, session_id);

CREATE TABLE messages (
    id BIGSERIAL PRIMARY KEY,
    session_id BIGINT NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
    sender_id BIGINT NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    message_type VARCHAR(16) NOT NULL DEFAULT 'text',
    content TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT messages_type_valid CHECK (message_type IN ('text', 'image', 'system')),
    CONSTRAINT messages_content_not_blank CHECK (CHAR_LENGTH(BTRIM(content)) > 0),
    CONSTRAINT messages_session_id_id_unique UNIQUE (session_id, id)
);

CREATE INDEX messages_session_id_id_desc_idx ON messages (session_id, id DESC);
CREATE INDEX messages_sender_id_created_at_idx ON messages (sender_id, created_at DESC);

ALTER TABLE sessions
ADD CONSTRAINT sessions_last_message_belongs_to_session
FOREIGN KEY (id, last_message_id)
REFERENCES messages (session_id, id)
ON DELETE RESTRICT;

CREATE TABLE user_session_read_state (
    user_id BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    session_id BIGINT NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
    last_read_message_id BIGINT,
    last_read_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (user_id, session_id),
    CONSTRAINT user_session_read_state_last_read_message_fk
        FOREIGN KEY (session_id, last_read_message_id)
        REFERENCES messages (session_id, id)
        ON DELETE RESTRICT
);

CREATE INDEX user_session_read_state_session_id_idx ON user_session_read_state (session_id);

CREATE TRIGGER trg_user_session_read_state_set_updated_at
BEFORE UPDATE ON user_session_read_state
FOR EACH ROW
EXECUTE FUNCTION set_updated_at();
