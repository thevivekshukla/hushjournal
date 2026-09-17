CREATE TABLE workspaces (
    id BLOB PRIMARY KEY NOT NULL,
    user_id BLOB NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    key_salt BLOB NOT NULL,
    encrypted_dek BLOB NOT NULL,
    passphrase_hint TEXT,
    mask BOOLEAN NOT NULL DEFAULT FALSE,
    total_workspace_size BIGINT NOT NULL DEFAULT 0,
    size_last_calculated_at DATETIME,
    created_at DATETIME NOT NULL DEFAULT (unixepoch()),
    updated_at DATETIME,
    CONSTRAINT workspaces_passphrase_hint_len CHECK (
        passphrase_hint IS NULL OR length(passphrase_hint) <= 255
    )
);

CREATE INDEX workspaces_user_id_idx ON workspaces (user_id);

CREATE TRIGGER workspaces_max_per_user_insert
BEFORE INSERT ON workspaces
FOR EACH ROW
WHEN (SELECT count(*) FROM workspaces WHERE user_id = NEW.user_id) >= 20
BEGIN
    SELECT RAISE(ABORT, 'a user cannot have more than 20 workspaces');
END;

CREATE TRIGGER workspaces_max_per_user_update
BEFORE UPDATE OF user_id ON workspaces
FOR EACH ROW
WHEN (
    SELECT count(*) FROM workspaces
    WHERE user_id = NEW.user_id AND id IS NOT NEW.id
) >= 20
BEGIN
    SELECT RAISE(ABORT, 'a user cannot have more than 20 workspaces');
END;

CREATE TRIGGER workspaces_set_updated_at
AFTER UPDATE ON workspaces
FOR EACH ROW
WHEN OLD.user_id IS NOT NEW.user_id
    OR OLD.name IS NOT NEW.name
    OR OLD.key_salt IS NOT NEW.key_salt
    OR OLD.encrypted_dek IS NOT NEW.encrypted_dek
    OR OLD.passphrase_hint IS NOT NEW.passphrase_hint
    OR OLD.mask IS NOT NEW.mask
    OR OLD.total_workspace_size IS NOT NEW.total_workspace_size
    OR OLD.size_last_calculated_at IS NOT NEW.size_last_calculated_at
BEGIN
    UPDATE workspaces SET updated_at = unixepoch() WHERE id = NEW.id;
END;
