backend := justfile_directory() / "backend"
frontend := justfile_directory() / "frontend"
spa_static := backend / "bin" / "hushjournal" / "static"

# Start the Vite dev server. /api is proxied to http://127.0.0.1:8000.
dev:
    pnpm --dir "{{frontend}}" dev

[working-directory: "backend"]
run:
    watchexec -r -e rs,toml,html cargo run

# Refresh the SQLx query cache against a migrated Postgres database.
[working-directory: "backend"]
prepare:
    #!/usr/bin/env bash
    set -euo pipefail
    export DATABASE_URL="${DATABASE_URL:-postgres://hushjournal:hushjournal@localhost:58417/hushjournal}"
    export SQLX_OFFLINE=false
    cargo sqlx database create
    cargo sqlx migrate run --source db/migrations
    cargo sqlx prepare --workspace

# Build the SvelteKit SPA, copy it into the API crate, then produce a release binary
# with the frontend embedded. Deep links fall back to adapter-static's 200.html.
[working-directory: "backend"]
build:
    pnpm --dir "{{frontend}}" install --frozen-lockfile
    pnpm --dir "{{frontend}}" build
    rm -rf "{{spa_static}}"
    mkdir -p "{{spa_static}}"
    cp -a "{{frontend}}/build/." "{{spa_static}}/"
    touch "{{spa_static}}/.gitkeep"
    test -f "{{spa_static}}/200.html"
    SQLX_OFFLINE=true cargo build --release
    @echo "release binary: ${CARGO_TARGET_DIR:-{{backend}}/target}/release/hushjournal"
