CREATE UNLOGGED TABLE kv_store (
    "key" VARCHAR(2048) PRIMARY KEY,
    "value" TEXT NOT NULL,
    expires TIMESTAMPTZ NOT NULL
);

CREATE INDEX kv_store_expires_idx ON kv_store (expires);
