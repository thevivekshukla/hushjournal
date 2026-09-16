# e2ejournal frontend

SvelteKit SPA for the encrypted journal. Styled with Tailwind CSS 4 and DaisyUI, with bits-ui for dialogs and menus.

```sh
pnpm install
pnpm dev
```

The UI is a prototype with local demo data. Sign-in does not call Google yet. `/api` is proxied to `http://127.0.0.1:8000` for when the real backend is wired in.

```sh
pnpm build
pnpm preview
```

`adapter-static` emits a `200.html` fallback for client-side routing.
