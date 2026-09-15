CREATE TABLE shelves (
    id UUID PRIMARY KEY DEFAULT uuidv7(),
    workspace_id UUID NOT NULL REFERENCES workspaces (id) ON DELETE CASCADE,
    name BYTEA NOT NULL,
    icon TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ
);

ALTER TABLE shelves
    ADD CONSTRAINT shelves_name_len_check CHECK (octet_length(name) <= 256);

CREATE INDEX shelves_workspace_id_idx ON shelves (workspace_id);

SELECT attach_updated_at_trigger('shelves');
