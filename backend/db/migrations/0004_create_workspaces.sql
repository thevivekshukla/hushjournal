CREATE TABLE workspaces (
    id UUID PRIMARY KEY DEFAULT uuidv7(),
    user_id UUID NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    key_salt BYTEA NOT NULL,
    encrypted_dek BYTEA NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ
);

CREATE INDEX workspaces_user_id_idx ON workspaces (user_id);

SELECT attach_updated_at_trigger('workspaces');
