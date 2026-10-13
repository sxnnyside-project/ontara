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

## Layout

- Features: `project`, `catalog`, `entity`, `relationship`, `editorial`, `asset`, `navigation`, `graph`, `matrix`, `timeline`, `consistency`, `dossier`, `history`, `search`, `assistant`, each with `api/`, `state/`, `ui/` and `index.ts`.
- `app/` composes the shell. `shared/` holds `backend`, `model`, `notice`, `revision`, `properties`, `i18n` and `ui`.
- Modules using runes are named `*.svelte.ts`. After a mutation call `revision.bump()`; data effects read `revision.value` to refresh.
- Texts live in `shared/i18n/texts/<group>.ts`, one file per area of the interface (`common`, `shell`, `project`, `entity`, `catalog`, `relationship`, `structure`, `library`, `graph`, `matrix`, `timeline`, `consistency`, `dossier`, `history`, `palette`, `data`, `onboarding`, `assistant`), each key holding its translations: `key: { en: '…', es: '…' }`. A missing language is a type error. A section lives in one group; `common` holds what several areas share. `texts/defaults.ts` holds the names of built-in catalog entries, shown in the interface language until the author renames them.
- Views are built with `ViewFrame` (header, toolbar, content, aside) and the compact controls in `shared/ui` (`InlineSelect`, `Segmented`, `SearchInput`, `SectionHeading`), so every view looks and behaves alike. An empty state offers the action that leaves it. The primary action of a view is always at the right of its header, the focus links (`ShowIn`) at the end of its toolbar, and side panels that depend on a selection are closable and absent without one. Icons are never placed in a rounded tile.
- Icons are MingCute classes written out in full in a file marked `// @unocss-include`.
- A new kind of dossier record is a row in `features/dossier/state/dossier-schema.ts` (and the adapter's table), its texts in `texts/dossier.ts`; the test fails until every field and choice has its words.
- Every edit the backend records has a name (`entity.update`); a new one needs a text in `historyLabels`.
- Accessibility is checked by `scripts/check-accessibility.ts` (part of `bun run lint`): icon-only buttons need a `title` or `aria-label`, no literal text in `title`, `aria-label` or `placeholder`, no `outline-none` without a focus style, and text colors must keep 4.5:1 on the surfaces in both themes. Text that is red uses `--color-crown-red-text`, not `--color-crown-red`.

## Tests

`bun test`, files in `tests/` mirroring `src/`. Mock the Tauri boundary with `mock.module('@tauri-apps/api/core', …)` before importing the module under test. `tests/setup/svelte-runes.ts` is preloaded so `*.svelte.ts` modules compile under `bun test`.
