CREATE TABLE shelves (
    id UUID PRIMARY KEY DEFAULT uuidv7(),
    workspace_id UUID NOT NULL REFERENCES workspaces (id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    icon TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ
);

CREATE INDEX shelves_workspace_id_idx ON shelves (workspace_id);

SELECT attach_updated_at_trigger('shelves');
