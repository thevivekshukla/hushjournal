# Production image for the HushJournal server.
# The SvelteKit build is embedded in the release binary (same result as `just build`).
#
# Required at runtime: APP_ORIGIN, DATABASE_URL.
# The process listens on 0.0.0.0:8000. Migrations run on startup.

FROM node:22.23-trixie-slim AS frontend

WORKDIR /src/frontend

ENV COREPACK_ENABLE_DOWNLOAD_PROMPT=0
RUN corepack enable && corepack prepare pnpm@12.6.0 --activate

COPY frontend/package.json frontend/pnpm-lock.yaml frontend/.npmrc ./
RUN pnpm install --frozen-lockfile

COPY frontend/ ./
RUN pnpm build

FROM rust:1.97.1-trixie AS backend

WORKDIR /src/backend

COPY backend/ ./
COPY --from=frontend /src/frontend/build/ ./bin/hushjournal/static/

RUN test -f bin/hushjournal/static/200.html
ENV SQLX_OFFLINE=true
RUN cargo build --release --locked --bin hushjournal

FROM debian:trixie-slim AS runtime

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --system --uid 65532 --no-create-home hushjournal

COPY --from=backend /src/backend/target/release/hushjournal /usr/local/bin/hushjournal

USER hushjournal

ENV HOST=0.0.0.0 \
    PORT=8000 \
    RUST_LOG=info,tower_http=info,sqlx=warn

EXPOSE 8000

ENTRYPOINT ["hushjournal"]
