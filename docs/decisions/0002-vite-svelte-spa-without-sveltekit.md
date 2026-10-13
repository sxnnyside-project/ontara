# 0002. Vite + Svelte single-page app, no SvelteKit

Status: accepted

## Context

The frontend runs inside a Tauri webview, loaded from local files. There is no server, no server-side rendering and no URL-driven routing across pages.

## Decision

Use Svelte 5 with Vite directly. The entry is `index.html` at the project root, mounting `src/app/main.ts`. Routing, when needed, is an in-app concern of the shell.

## Consequences

- No server-side concepts (load functions, adapters, prerendering) to configure or avoid.
- Fewer moving parts between the source and the Tauri `frontendDist`.
- If routing grows complex, a client-side router is added deliberately as its own decision.
