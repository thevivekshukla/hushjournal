HushJournal is an end-to-end encrypted Journal web app. It uses AES-GCM-SIV to encrypt the content on the client-side, server sees only the ciphered text.

> HushJournal is made using LLM but is not vibe coded. It is opinionated and built to serve the specific purpose of secure and priviate journal.

## Features

- Using very secure encryption method: AES-GCM-SIV.
- Multiple journals per account.
- Each Journal has their own passphrase.
- A Journal holds multiple Notebooks. Each Notebook holds multiple Entries.
- Add template to Notebook, so when you create an Entry it is pre-filled with template data.
- Mask a Journal so that Notebook and Entry titles are masked when not open. Hides from prying eyes
- Give each Journal a different look, apply different themes to each Journal from the set of 20 themes

## Getting Started

**Docker** 

The easiest way to run HushJournal is via Docker. HushJournal requires a postgres database version 17 or above.

```bash
docker run --rm \
    -e DATABASE_URL="postgres://hushjournal:hushjournal@172.17.0.1:5432/hushjournal" \
    -e APP_ORIGIN="http://127.0.0.1:8000" \
    -p 8000:8000 \
    thevivekshukla/hushjournal:latest
```

**Docker Compose**

Use docker compose to run both postgres server and HushJournal server in same network.

```yml
services:
  db:
    image: postgres:18
    environment:
      TZ: UTC
      PGTZ: UTC
      POSTGRES_USER: hushjournal
      POSTGRES_PASSWORD: hushjournal
      POSTGRES_DB: hushjournal
    command: ["postgres", "-c", "timezone=UTC"]
    volumes:
      - hushjournal_postgres:/var/lib/postgresql
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U hushjournal -d hushjournal"]
      interval: 5s
      timeout: 5s
      retries: 5

  hushjournal:
    image: thevivekshukla/hushjournal:latest
    depends_on:
      db:
        condition: service_healthy
    environment:
      DATABASE_URL: postgres://hushjournal:hushjournal@db:5432/hushjournal
      APP_ORIGIN: http://127.0.0.1:8000
    ports:
      - "8000:8000"

volumes:
  hushjournal_postgres:
```

## Screenshots

## LICENSE
