-- created_at / updated_at are timestamptz in UTC.
-- created_at DEFAULT now(); updated_at stays NULL until a real UPDATE.
-- After CREATE TABLE, attach the trigger with:
--   SELECT attach_updated_at_trigger('table_name');

DO $$
BEGIN
    EXECUTE format(
        'ALTER DATABASE %I SET timezone TO %L',
        current_database(),
        'UTC'
    );
END
$$;

SET timezone TO 'UTC';

CREATE OR REPLACE FUNCTION set_updated_at()
RETURNS trigger
LANGUAGE plpgsql
AS $$
BEGIN
    -- Ignore application writes to updated_at; only stamp it when other columns change.
    NEW.updated_at = OLD.updated_at;
    IF NEW IS DISTINCT FROM OLD THEN
        NEW.updated_at = CURRENT_TIMESTAMP;
    END IF;
    RETURN NEW;
END;
$$;

CREATE OR REPLACE FUNCTION attach_updated_at_trigger(table_name regclass)
RETURNS void
LANGUAGE plpgsql
AS $$
BEGIN
    EXECUTE format(
        'CREATE OR REPLACE TRIGGER set_updated_at
            BEFORE UPDATE ON %s
            FOR EACH ROW
            EXECUTE FUNCTION set_updated_at()',
        table_name
    );
END;
$$;
