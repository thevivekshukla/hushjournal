CREATE TABLE IF NOT EXISTS journals (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    key_salt BYTEA NOT NULL,
    encrypted_dek BYTEA NOT NULL,
    passphrase_hint TEXT,
    mask BOOLEAN NOT NULL DEFAULT false,
    theme TEXT NOT NULL DEFAULT '',
    total_journal_size BIGINT NOT NULL DEFAULT 0,
    size_last_calculated_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ,
    CONSTRAINT journals_passphrase_hint_len CHECK (
        passphrase_hint IS NULL OR char_length(passphrase_hint) <= 255
    ),
    CONSTRAINT journals_theme_values CHECK (
        theme IN (
            '',
            'silk',
            'cupcake',
            'bumblebee',
            'emerald',
            'corporate',
            'nord',
            'lemonade',
            'winter',
            'caramellatte',
            'retro',
            'dim',
            'forest',
            'dracula',
            'night',
            'coffee',
            'synthwave',
            'abyss',
            'luxury',
            'halloween',
            'sunset'
        )
    )
);

CREATE INDEX IF NOT EXISTS journals_user_id_idx ON journals (user_id);

SELECT attach_updated_at_trigger('journals');
