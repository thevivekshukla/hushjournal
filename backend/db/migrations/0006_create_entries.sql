CREATE TABLE entries (
    id UUID PRIMARY KEY DEFAULT uuidv7(),
    shelf_id UUID NOT NULL REFERENCES shelves (id) ON DELETE CASCADE,
    title TEXT NOT NULL,
    tags TEXT[] NOT NULL DEFAULT '{}',
    content BYTEA NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ
);

CREATE INDEX entries_shelf_id_idx ON entries (shelf_id);

SELECT attach_updated_at_trigger('entries');
