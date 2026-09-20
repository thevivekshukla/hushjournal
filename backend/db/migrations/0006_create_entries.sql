CREATE TABLE IF NOT EXISTS entries (
    id UUID PRIMARY KEY,
    notebook_id UUID NOT NULL REFERENCES notebooks (id) ON DELETE CASCADE,
    title BYTEA NOT NULL,
    content BYTEA NOT NULL,
    total_size BIGINT NOT NULL DEFAULT 0,
    entry_date DATE NOT NULL DEFAULT CURRENT_DATE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ
);

ALTER TABLE entries
    DROP CONSTRAINT IF EXISTS entries_title_len_check,
    DROP CONSTRAINT IF EXISTS entries_content_len_check;
ALTER TABLE entries
    ADD CONSTRAINT entries_title_len_check CHECK (octet_length(title) <= 1024),
    ADD CONSTRAINT entries_content_len_check CHECK (octet_length(content) <= 5242880);

CREATE OR REPLACE FUNCTION set_entry_total_size()
RETURNS trigger
LANGUAGE plpgsql
AS $$
BEGIN
    NEW.total_size = octet_length(NEW.title) + octet_length(NEW.content);
    RETURN NEW;
END;
$$;

DROP TRIGGER IF EXISTS set_entry_total_size ON entries;
CREATE TRIGGER set_entry_total_size
    BEFORE INSERT OR UPDATE ON entries
    FOR EACH ROW
    EXECUTE FUNCTION set_entry_total_size();

CREATE INDEX IF NOT EXISTS entries_notebook_id_idx ON entries (notebook_id);

SELECT attach_updated_at_trigger('entries');
