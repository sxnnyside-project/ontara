# Frontend (`src/`)

Svelte 5 (runes) + TypeScript, bundled by Vite, run by Bun. A single-page app rendered inside the Tauri webview. No SvelteKit.

## Rules

- The frontend renders and orchestrates; it does not decide domain rules. Anything that validates, derives or persists narrative data goes through a Tauri command.
- Backend calls live in `features/<feature>/api/`. Each wrapper has a typed result that mirrors the Rust DTO and names it in a doc comment.
- A feature exposes one public entry point, `index.ts`. Other features import from `@/features/<feature>`, never from its internals.
- Dependency direction: `app` → `features` → `shared`.
- Files and folders are kebab-case, including `.svelte` files (`app-info-card.svelte`). Named exports; default exports only for framework or config files.
- Biome formats and lints; `.svelte` files are formatted by Biome, and unused code in them is reported by `svelte-check` (Biome cannot see template usage).
- TypeScript is pinned to 6.0 because `svelte-check` does not support TypeScript 7 without a second, experimental toolchain. Revisit when it does.

## Tests

`bun test`, files in `tests/` mirroring `src/`. Mock the Tauri boundary with `mock.module('@tauri-apps/api/core', …)` before importing the module under test.
