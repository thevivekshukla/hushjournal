CREATE TABLE entries (
    id BLOB PRIMARY KEY NOT NULL,
    shelf_id BLOB NOT NULL REFERENCES shelves (id) ON DELETE CASCADE,
    title BLOB NOT NULL,
    content BLOB NOT NULL,
    total_size BIGINT NOT NULL GENERATED ALWAYS AS (length(title) + length(content)) STORED,
    created_at DATETIME NOT NULL DEFAULT (unixepoch()),
    updated_at DATETIME,
    CONSTRAINT entries_title_len_check CHECK (length(title) <= 1024),
    CONSTRAINT entries_content_len_check CHECK (length(content) <= 5242880)
);

CREATE INDEX entries_shelf_id_idx ON entries (shelf_id);

CREATE TRIGGER entries_set_updated_at
AFTER UPDATE ON entries
FOR EACH ROW
WHEN OLD.shelf_id IS NOT NEW.shelf_id
    OR OLD.title IS NOT NEW.title
    OR OLD.content IS NOT NEW.content
BEGIN
    UPDATE entries SET updated_at = unixepoch() WHERE id = NEW.id;
END;
