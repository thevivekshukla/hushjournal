CREATE TABLE workspaces (
    id UUID PRIMARY KEY DEFAULT uuidv7(),
    user_id UUID NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    key_salt BYTEA NOT NULL,
    encrypted_dek BYTEA NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ
);

CREATE INDEX workspaces_user_id_idx ON workspaces (user_id);

CREATE OR REPLACE FUNCTION check_workspaces_max_per_user()
RETURNS trigger
LANGUAGE plpgsql
AS $$
BEGIN
    IF (
        SELECT count(*) FROM workspaces
        WHERE user_id = NEW.user_id
          AND id IS DISTINCT FROM NEW.id
    ) >= 20 THEN
        RAISE EXCEPTION 'a user cannot have more than 20 workspaces'
            USING ERRCODE = 'check_violation';
    END IF;
    RETURN NEW;
END;
$$;

CREATE TRIGGER workspaces_max_per_user
    BEFORE INSERT OR UPDATE OF user_id ON workspaces
    FOR EACH ROW
    EXECUTE FUNCTION check_workspaces_max_per_user();

SELECT attach_updated_at_trigger('workspaces');
