CREATE TABLE entries (
    id UUID PRIMARY KEY DEFAULT uuidv7(),
    shelf_id UUID NOT NULL REFERENCES shelves (id) ON DELETE CASCADE,
    title BYTEA NOT NULL,
    content BYTEA NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ
);

ALTER TABLE entries
    ADD CONSTRAINT entries_title_len_check CHECK (octet_length(title) <= 1024),
    ADD CONSTRAINT entries_content_len_check CHECK (octet_length(content) <= 5242880);

CREATE INDEX entries_shelf_id_idx ON entries (shelf_id);

SELECT attach_updated_at_trigger('entries');
