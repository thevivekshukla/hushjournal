CREATE TABLE kv_store (
    "key" TEXT PRIMARY KEY,
    "value" TEXT NOT NULL,
    expires BIGINT NOT NULL
);

CREATE INDEX kv_store_expires_idx ON kv_store (expires);
