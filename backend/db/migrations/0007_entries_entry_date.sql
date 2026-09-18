ALTER TABLE entries
    ADD COLUMN entry_date DATE NOT NULL DEFAULT CURRENT_DATE;

UPDATE entries
    SET entry_date = (created_at AT TIME ZONE 'UTC')::date;
