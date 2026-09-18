ALTER TABLE users ADD COLUMN username TEXT;

CREATE UNIQUE INDEX users_username_key ON users (username) WHERE username IS NOT NULL;

DROP TRIGGER users_set_updated_at;
CREATE TRIGGER users_set_updated_at
AFTER UPDATE ON users
FOR EACH ROW
WHEN OLD.name IS NOT NEW.name
    OR OLD.username IS NOT NEW.username
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
