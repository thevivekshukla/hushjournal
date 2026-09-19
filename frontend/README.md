# e2ejournal frontend

SvelteKit SPA for the encrypted journal. Styled with Tailwind CSS 4 and DaisyUI, with bits-ui for dialogs and menus.

```sh
pnpm install
pnpm dev
```

The API must be running on `http://127.0.0.1:8000`. Vite proxies `/api` there. `APP_ORIGIN=http://127.0.0.1:5173` is required on the API so Google login callbacks and the post-login redirect return to this app.

Notebook names, entry titles, and entry content are encrypted with AES-256-GCM-SIV in the browser before they are sent. The journal passphrase never leaves the device.

```sh
pnpm build
pnpm preview
```

`adapter-static` emits a `200.html` fallback for client-side routing.
