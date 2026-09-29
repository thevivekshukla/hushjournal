ALTER TABLE notebooks
    ADD COLUMN IF NOT EXISTS template_entry_content BYTEA;

ALTER TABLE notebooks
    DROP CONSTRAINT IF EXISTS notebooks_template_entry_content_len_check;
ALTER TABLE notebooks
    ADD CONSTRAINT notebooks_template_entry_content_len_check CHECK (
        template_entry_content IS NULL
        OR (
            octet_length(template_entry_content) > 0
            AND octet_length(template_entry_content) <= 5 * 1024 * 1024
        )
    );
