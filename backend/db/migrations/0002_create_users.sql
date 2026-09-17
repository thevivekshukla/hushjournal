CREATE TABLE users (
    id BLOB PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    email TEXT,
    is_email_verified BOOLEAN NOT NULL DEFAULT FALSE,
    email_verified_at DATETIME,
    password_hash TEXT,
    google_email TEXT,
    google_account_id TEXT,
    google_avatar_url TEXT,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    last_login_at DATETIME,
    created_at DATETIME NOT NULL DEFAULT (unixepoch()),
    updated_at DATETIME
);

CREATE UNIQUE INDEX users_email_key ON users (email) WHERE email IS NOT NULL;
CREATE UNIQUE INDEX users_google_account_id_key ON users (google_account_id)
    WHERE google_account_id IS NOT NULL;

CREATE TRIGGER users_set_updated_at
AFTER UPDATE ON users
FOR EACH ROW
WHEN OLD.name IS NOT NEW.name
    OR OLD.email IS NOT NEW.email
    OR OLD.is_email_verified IS NOT NEW.is_email_verified
    OR OLD.email_verified_at IS NOT NEW.email_verified_at
    OR OLD.password_hash IS NOT NEW.password_hash
    OR OLD.google_email IS NOT NEW.google_email
    OR OLD.google_account_id IS NOT NEW.google_account_id
    OR OLD.google_avatar_url IS NOT NEW.google_avatar_url
    OR OLD.is_active IS NOT NEW.is_active
    OR OLD.last_login_at IS NOT NEW.last_login_at
BEGIN
    UPDATE users SET updated_at = unixepoch() WHERE id = NEW.id;
END;
