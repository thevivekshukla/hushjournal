CREATE TABLE shelves (
    id UUID PRIMARY KEY DEFAULT uuidv7(),
    workspace_id UUID NOT NULL REFERENCES workspaces (id) ON DELETE CASCADE,
    name BYTEA NOT NULL,
    icon TEXT,
    total_shelf_size BIGINT NOT NULL DEFAULT 0,
    size_last_calculated_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ
);

ALTER TABLE shelves
    ADD CONSTRAINT shelves_name_len_check CHECK (octet_length(name) <= 256);

CREATE INDEX shelves_workspace_id_idx ON shelves (workspace_id);

CREATE OR REPLACE FUNCTION check_shelves_max_per_workspace()
RETURNS trigger
LANGUAGE plpgsql
AS $$
BEGIN
    IF (
        SELECT count(*) FROM shelves
        WHERE workspace_id = NEW.workspace_id
          AND id IS DISTINCT FROM NEW.id
    ) >= 100 THEN
        RAISE EXCEPTION 'a workspace cannot have more than 100 shelves'
            USING ERRCODE = 'check_violation';
    END IF;
    RETURN NEW;
END;
$$;

CREATE TRIGGER shelves_max_per_workspace
    BEFORE INSERT OR UPDATE OF workspace_id ON shelves
    FOR EACH ROW
    EXECUTE FUNCTION check_shelves_max_per_workspace();

SELECT attach_updated_at_trigger('shelves');
