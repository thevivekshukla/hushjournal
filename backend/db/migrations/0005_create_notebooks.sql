CREATE TABLE IF NOT EXISTS notebooks (
    id UUID PRIMARY KEY DEFAULT uuidv7(),
    journal_id UUID NOT NULL REFERENCES journals (id) ON DELETE CASCADE,
    name BYTEA NOT NULL,
    icon TEXT,
    total_notebook_size BIGINT NOT NULL DEFAULT 0,
    size_last_calculated_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ
);

ALTER TABLE notebooks
    DROP CONSTRAINT IF EXISTS notebooks_name_len_check;
ALTER TABLE notebooks
    ADD CONSTRAINT notebooks_name_len_check CHECK (octet_length(name) <= 256);

CREATE INDEX IF NOT EXISTS notebooks_journal_id_idx ON notebooks (journal_id);

CREATE OR REPLACE FUNCTION check_notebooks_max_per_journal()
RETURNS trigger
LANGUAGE plpgsql
AS $$
BEGIN
    IF (
        SELECT count(*) FROM notebooks
        WHERE journal_id = NEW.journal_id
          AND id IS DISTINCT FROM NEW.id
    ) >= 100 THEN
        RAISE EXCEPTION 'a journal cannot have more than 100 notebooks'
            USING ERRCODE = 'check_violation';
    END IF;
    RETURN NEW;
END;
$$;

DROP TRIGGER IF EXISTS notebooks_max_per_journal ON notebooks;
CREATE TRIGGER notebooks_max_per_journal
    BEFORE INSERT OR UPDATE OF journal_id ON notebooks
    FOR EACH ROW
    EXECUTE FUNCTION check_notebooks_max_per_journal();

SELECT attach_updated_at_trigger('notebooks');
