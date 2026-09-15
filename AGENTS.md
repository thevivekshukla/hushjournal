# e2ejournal

End-to-end encrypted journal. The client encrypts with AES-256-GCM-SIV. The server stores ciphertext only and must never see, log, or decrypt user content.

Users sign in with Google OAuth2. A user has many Workspaces; a Workspace has one or more Shelves; a Shelf has many entries. An entry is encrypted `title` and encrypted `content`. New-entry titles default to today's date in this form: `7 Sep 2026` (set and encrypted on the client). Shelf `name` is also ciphertext.

Crypto material lives on the workspace, not the user: `key_salt` and `encrypted_dek`. The server stores these as opaque blobs and must never try to unwrap them. Shelf rows belong to a workspace (`workspace_id`); entry rows belong to a shelf (`shelf_id`). Do not store plaintext for `workspaces.key_salt`, `workspaces.encrypted_dek`, `shelves.name`, `entries.title`, or `entries.content`.

## Backend

Rust workspace under `backend/`. Edition 2024. Stack: Axum, SQLx, Postgres.

```
backend/
  bin/e2ejournal/   # API binary; keep Axum routes in src/main.rs
  errors/           # AppError and HTTP error mapping
  db/               # pool, AppState, SQLx migrations
  utils/            # config, Axum session extractors
```

- `cargo run` from `backend/` should start the API (`default-members` is `bin/e2ejournal`).
- Shared code goes in `errors`, `db`, or `utils` — do not grow a kitchen-sink binary crate.
- Keep REST route handlers in `bin/e2ejournal/src/main.rs` until there is a strong reason to split them.
- Default API bind: `127.0.0.1:8000` (`HOST` / `PORT`). Do not change the default port to 3000.

## Postgres

- Image: `postgres:18` in `backend/docker-compose.yml`.
- Do not publish host port `5432`. Use a non-default port (currently `58417:5432`) so it does not clash with other local Postgres instances.
- Timezone is UTC: database `timezone=UTC`, `timestamptz` columns, `now()` / `CURRENT_TIMESTAMP`.
- Primary keys are `UUID` with `DEFAULT uuidv7()` (Postgres 18). Do not use `gen_random_uuid()` or UUIDv4. Omit `id` on insert unless you have a reason to pass one.
- Tables that need timestamps use:

```sql
created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
updated_at TIMESTAMPTZ
```

- `updated_at` is NULL on insert. `set_updated_at()` sets it only when other columns change. After `CREATE TABLE`, attach it with `SELECT attach_updated_at_trigger('table_name');`. Do not set `updated_at` in application code.
- `entries.total_size` is `BIGINT NOT NULL DEFAULT 0` and is set by `set_entry_total_size` on INSERT/UPDATE to `octet_length(title) + octet_length(content)`. Do not set it in application code.
- `shelves.total_shelf_size` is `BIGINT NOT NULL DEFAULT 0`. `shelves.size_last_calculated_at` is `TIMESTAMPTZ` NULL. Both are maintained by application cron, not triggers. Recalculate a shelf when any of its entries has `created_at` or `updated_at` after `size_last_calculated_at` (treat NULL as never calculated).
- Max 20 workspaces per user (`workspaces_max_per_user`) and 100 shelves per workspace (`shelves_max_per_workspace`). Enforced by BEFORE INSERT/UPDATE triggers; CHECK cannot see other rows.
- Migrations live in `backend/db/migrations/` and are applied on API startup via `sqlx::migrate!()`.
- Sessions, cookies, and other short-lived scratch data go in the UNLOGGED `kv_store` table via `db::PgStore`. Do not add Redis. Do not store journal content, `key_salt`, or `encrypted_dek` there — UNLOGGED tables skip WAL and can be lost on crash.

## Sessions

- Auth is cookie sessions, not bearer tokens. Put a random session id in an HttpOnly `session` cookie; never store the raw id. SHA-256 the id and use that digest as the `PgStore` key (`session:<hex>`).
- Attach `user_id` and other session values with `Session::attach` / `Session::remove`. After `attach` on a new session, send `Set-Cookie` via `Session::cookie(cookie_secure)` (`COOKIE_SECURE`, default false on localhost).
- Handlers extract `utils::Session` (optional login) or `utils::UserId` (required login, 401 if missing) through `FromRequestParts`. Do not read the raw cookie in handlers.

## Product constraints

- Never add server-side encryption, decryption, or plaintext indexing of shelf names, entry titles, or entry content.
- Prefer storing encrypted blobs as the client sent them.
- Ciphertext size limits (`octet_length`): shelf `name` 256 bytes, entry `title` 1 KiB, entry `content` 5 MiB.
