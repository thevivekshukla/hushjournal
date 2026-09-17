CREATE TABLE shelves (
    id BLOB PRIMARY KEY NOT NULL,
    workspace_id BLOB NOT NULL REFERENCES workspaces (id) ON DELETE CASCADE,
    name BLOB NOT NULL,
    icon TEXT,
    total_shelf_size BIGINT NOT NULL DEFAULT 0,
    size_last_calculated_at DATETIME,
    created_at DATETIME NOT NULL DEFAULT (unixepoch()),
    updated_at DATETIME,
    CONSTRAINT shelves_name_len_check CHECK (length(name) <= 256)
);

CREATE INDEX shelves_workspace_id_idx ON shelves (workspace_id);

CREATE TRIGGER shelves_max_per_workspace_insert
BEFORE INSERT ON shelves
FOR EACH ROW
WHEN (SELECT count(*) FROM shelves WHERE workspace_id = NEW.workspace_id) >= 100
BEGIN
    SELECT RAISE(ABORT, 'a workspace cannot have more than 100 shelves');
END;

CREATE TRIGGER shelves_max_per_workspace_update
BEFORE UPDATE OF workspace_id ON shelves
FOR EACH ROW
WHEN (
    SELECT count(*) FROM shelves
    WHERE workspace_id = NEW.workspace_id AND id IS NOT NEW.id
) >= 100
BEGIN
    SELECT RAISE(ABORT, 'a workspace cannot have more than 100 shelves');
END;

CREATE TRIGGER shelves_set_updated_at
AFTER UPDATE ON shelves
FOR EACH ROW
WHEN OLD.workspace_id IS NOT NEW.workspace_id
    OR OLD.name IS NOT NEW.name
    OR OLD.icon IS NOT NEW.icon
    OR OLD.total_shelf_size IS NOT NEW.total_shelf_size
    OR OLD.size_last_calculated_at IS NOT NEW.size_last_calculated_at
BEGIN
    UPDATE shelves SET updated_at = unixepoch() WHERE id = NEW.id;
END;
