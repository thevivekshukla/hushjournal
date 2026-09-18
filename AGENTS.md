# e2ejournal

End-to-end encrypted journal. The client encrypts with AES-256-GCM-SIV. The server stores ciphertext only and must never see, log, or decrypt user content.

Users sign in with Google OAuth2 or a username and password. A user has many Journals; a Journal has one or more Notebooks; a Notebook has many entries. An entry is encrypted `title` and encrypted `content`. New-entry titles default to today's date in this form: `7 Sep 2026` (set and encrypted on the client). Notebook `name` is also ciphertext.

Crypto material lives on the journal, not the user: `key_salt` and `encrypted_dek`. The server stores these as opaque blobs and must never try to unwrap them. Notebook rows belong to a journal (`journal_id`); entry rows belong to a notebook (`notebook_id`). Do not store plaintext for `journals.key_salt`, `journals.encrypted_dek`, `notebooks.name`, `entries.title`, or `entries.content`. `journals.passphrase_hint` is optional plaintext (max 255 chars) so it can be shown before unlock; empty PATCH value clears it. `journals.mask` is a plaintext boolean (default false): when true, the client hides inactive notebook names (icons stay) and inactive entry titles. `entries.entry_date` is a plaintext calendar date (default creation date) shown in the editor.

## Backend

Rust workspace under `backend/`. Edition 2024. Stack: Axum, SQLx, Postgres.

```
backend/
  bin/e2ejournal/   # API binary; mount crate routers from src/main.rs
  errors/           # AppError and HTTP error mapping
  db/               # pool, AppState, SQLx migrations
  utils/            # config, Axum session extractors, shared reqwest client
  user/             # user row types and Postgres queries
  user_json/        # Axum JSON handlers / router for user and auth APIs
  journal/          # journal, notebook, and entry row types and Postgres queries
  journal_json/     # Axum JSON handlers / router for journals, notebooks, and entries
```

- `cargo run` from `backend/` should start the API (`default-members` is `bin/e2ejournal`).
- Shared outbound HTTP uses `utils::reqwest_client()` (a process-wide `reqwest::Client`). Do not create additional reqwest clients.
- Domain DB access lives in entity crates (`user`, `journal`). `journal` covers journals, notebooks, and entries together. REST handlers for those entities live in `*_json` crates and are `.nest("/api", ...)` from `bin/e2ejournal/src/main.rs`. `*_json` routers must not include the `/api` prefix themselves. Keep `/health`, `GET /`, and server wiring in `main.rs` (health stays outside `/api`).
- Journal REST: `/journals`, `/journals/{id}/notebooks`, `/notebooks/{id}/entries` for collections; `/journals/{id}`, `/notebooks/{id}`, `/entries/{id}` for a single row. All require a session. BYTEA ciphertext is JSON standard-base64. `entries.entry_date` is plaintext ISO `YYYY-MM-DD` (editable; omitted create uses the database default). List entries omit `content` and return a cursor page `{ entries, next_cursor }` ordered by `id` (UUIDv7). Query params: `cursor` (entry id), `order=asc|desc` (default desc), `limit` (1–100, default 50). `GET /entries/{id}` returns the body. Do not log ciphertext.
- Write SQLx queries in-place at the call site. Do not abstract SQL into shared consts, macros, or concatenated column lists. If a query is too long for a normal editor width, break it across multiple lines in a raw string (`r#"..."#`). Keep short queries on one line.
- Always use the type-checked SQLx macros (`query!`, `query_as!`, `query_scalar!`). Do not use `sqlx::query()`, `query_as()`, or `query_scalar()`. After adding or changing queries, run `just prepare` from `backend/` against a migrated Postgres database and commit the `.sqlx` cache.
- Default API bind: `127.0.0.1:8000` (`HOST` / `PORT`). Do not change the default port to 3000.
- `GOOGLE_LOGIN_OAUTH2` is required: `client_id,client_secret` (comma-separated, first comma splits). `GOOGLE_OAUTH_REDIRECT_URI` is required and must match the Google Cloud OAuth client redirect URI (for local: `http://127.0.0.1:8000/api/auth/google/callback`).
- Google OAuth is the authorization-code flow. Start at `GET /api/auth/google` (optional `next` query, relative path only), callback at `GET /api/auth/google/callback`. After login, redirect to `next` or `/journals`. `APP_ORIGIN` (optional, e.g. `http://127.0.0.1:5173`) prefixes that path so the SPA receives the session; omit it when the app is served from the API origin. Never log OAuth codes, tokens, or client secrets.
- Do not add CORS. The SPA is same-origin: Vite proxies `/api` in development, and production embeds the Vite build in the API binary. `APP_ORIGIN` is only for the OAuth redirect, not a CORS allowlist; omit it when the SPA is served from the API origin. Cross-origin browsers must not be able to call the API.
- Production static files live in `bin/e2ejournal/static` (gitignored build output). `just build` from `backend/` runs the frontend build, copies it there, then `cargo build --release`. The binary serves hashed assets from that tree and uses SvelteKit's `200.html` as the GET fallback for client routes. Keep `/health` and `/api` ahead of that fallback. `GET /` always 307s to `/login`.

## Postgres

- Image: `postgres:18` in `backend/docker-compose.yml`.
- Mount the data volume at `/var/lib/postgresql` (Postgres 18 image), not `/var/lib/postgresql/data`.
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
- `entries.entry_date` is plaintext `DATE NOT NULL DEFAULT CURRENT_DATE`. The client may send it on create/PATCH; omit it to keep the default. Do not encrypt it.
- `notebooks.total_notebook_size` is `BIGINT NOT NULL DEFAULT 0`. `notebooks.size_last_calculated_at` is `TIMESTAMPTZ` NULL. Both are maintained by application cron, not triggers. Recalculate a notebook when any of its entries has `created_at` or `updated_at` after `size_last_calculated_at` (treat NULL as never calculated).
- `journals.total_journal_size` is `BIGINT NOT NULL DEFAULT 0`. `journals.size_last_calculated_at` is `TIMESTAMPTZ` NULL. Same cron, not triggers. Recalculate a journal when any of its notebooks has `created_at`, `updated_at`, or `size_last_calculated_at` after the journal's `size_last_calculated_at` (treat NULL as never calculated). Do not set either column in application CRUD. Spawn both jobs from `bin/e2ejournal` with `tokio::spawn` (notebooks first, then journals, every 60s).
- Max 20 journals per user (`journals_max_per_user`) and 100 notebooks per journal (`notebooks_max_per_journal`). Enforced by BEFORE INSERT/UPDATE triggers; CHECK cannot see other rows.
- Migrations live in `backend/db/migrations/` and are applied on API startup via `sqlx::migrate!()`.
- Sessions, cookies, and other short-lived scratch data go in the UNLOGGED `kv_store` table via `db::PgStore`. Do not add Redis. Do not store journal content, `key_salt`, or `encrypted_dek` there — UNLOGGED tables skip WAL and can be lost on crash.
- Backup with `e2ejournal db-backup`. It runs `pg_dump -Fc` to `e2ejournal-{YYYYMMDDTHHMMSSZ}.dump` in the current directory. `--path FILE` writes to that file instead and refuses to overwrite.
- `e2ejournal env` writes a sample `.env` in the current directory (same defaults as `backend/.env.example`). If `.env` already exists it prints a message and does not overwrite.

## Sessions

- Auth is cookie sessions, not bearer tokens. Put a random session id in an HttpOnly `session` cookie; never store the raw id. SHA-256 the id and use that digest as the `PgStore` key (`session:<hex>`).
- Attach `user_id` and other session values with `Session::attach` / `Session::remove`. After `attach` on a new session, send `Set-Cookie` via `Session::cookie(cookie_secure)` (`COOKIE_SECURE`, default false on localhost). On logout / account delete, destroy the session and send `Session::removal_cookie(cookie_secure)`.
- Handlers extract `utils::Session` (optional login) or `utils::UserId` (required login, 401 if missing) through `FromRequestParts`. Do not read the raw cookie in handlers.
- Google login upserts by `google_account_id` (create if missing, update `google_email` / `google_avatar_url` / `last_login_at` on repeat login). Do not overwrite a user-edited `name` on subsequent Google logins. Username/password signup is `POST /auth/signup` `{ username, password }`; login is `POST /auth/login` with the same body. Usernames are unique (stored lowercase, 3–32 `[a-z][a-z0-9_]*`). Hash passwords with Argon2id into `users.password_hash`; never return or log the password or hash. Google-only rows may have null `username` / `password_hash`. Inactive users (`is_active = false`) must not be signed in. `DELETE /api/user` deletes the `users` row (journals/notebooks/entries cascade). Profile edit (`PATCH /api/user`) may change `name` only for now.

## Product constraints

- Never add server-side encryption, decryption, or plaintext indexing of notebook names, entry titles, or entry content.
- Prefer storing encrypted blobs as the client sent them.
- Ciphertext size limits (`octet_length`): notebook `name` 256 bytes, entry `title` 1 KiB, entry `content` 5 MiB. Passphrase hint is plaintext `char_length` 255.

## Agent

- Do not use browser tools or CDP (screenshots, snapshots, clicks, `Runtime.evaluate`) unless the user explicitly asks. UI work is code-only until then.

## Frontend

SvelteKit SPA under `frontend/`. Full client render: `adapter-static` with `fallback: '200.html'`, and `ssr = false` in the root layout. Stack: Svelte 5 runes, Tailwind CSS 4, DaisyUI 5, bits-ui (dialogs and menus), Iconify Tailwind icons, Figtree + Literata.

```
frontend/
  src/routes/           # pages; no +server.js / +page.server.js
  src/lib/components/   # bits-ui wrappers and editor chrome
  src/lib/*.svelte.ts   # client session and encrypted journal state
```

- Use pnpm for all frontend package manager commands (`pnpm install`, `pnpm add`, `pnpm dev`, `pnpm check`). Do not use npm or yarn. Keep `pnpm-lock.yaml`; do not add `package-lock.json`.
- `pnpm dev` from `frontend/` (Vite, default 5173). `/api` is proxied to `http://127.0.0.1:8000`. Production: `just build` from `backend/` embeds `frontend/build` into the API binary. `/` always redirects to `/login` (SvelteKit `load` in dev; Axum `GET /` in production). The login page has Google OAuth plus username/password Sign in and Sign up tabs.
- Themes: DaisyUI `silk` (light) and `dim` (dark). Persist the choice in `localStorage` as `theme`.
- The SPA talks to the REST API through the Vite `/api` proxy. Encrypt notebook names, entry titles, and entry content with AES-256-GCM-SIV on the client before upload. Never send the journal passphrase or plaintext journal content to the API. Keep the unwrapped DEK in memory only.
- Changing a journal passphrase re-wraps the existing DEK on the client with a new salt and PATCHes `key_salt` and `encrypted_dek` together. Do not rotate the DEK or re-encrypt notes. The server must not see the old or new passphrase. Routes: `/journals` lists journals; `/journals/[journalId]` is the unlocked editor.
- Default new-entry title is today's date: `7 Sep 2026`. Show `entry_date` to the left of the Saved/Saving status in the editor; clicking it opens a calendar to change the date.
- Use bits-ui for dialogs, dropdowns, and other focus-trap widgets. Use DaisyUI classes for visual styling. Prefer Iconify `icon-[lucide--…]` classes over per-icon Svelte packages.
