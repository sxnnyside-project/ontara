# Ontara

Local-first desktop environment for modeling and maintaining fictional universes. Product definition: `docs/architecture.md` and the domain terms in `docs/glossary.md`.

## Topology

Non-monolithic, polyglot, one repository, two language boundaries:

| Boundary | Location | Language | Own context |
|---|---|---|---|
| Frontend | `src/`, `tests/`, root configs | TypeScript + Svelte 5 on Bun and Vite | `src/CLAUDE.md` |
| Backend | `src-tauri/` | Rust (Cargo workspace) | `src-tauri/CLAUDE.md` |

The Rust workspace root is `src-tauri/Cargo.toml`: the root package is the Tauri adapter, domain logic lives in `src-tauri/crates/`.

## Commands

Everything runs through `just` from the repo root. `just check` is the gate; CI and the pre-commit hook call it.

| Command | Does |
|---|---|
| `just install` | Install JS and Rust dependencies, tools and git hooks |
| `just dev` | Run the desktop app with hot reload |
| `just build` | Build the frontend and the release binary (no installers) |
| `just test` | Frontend (`bun test`) and Rust (`cargo test`) suites |
| `just typecheck` | `svelte-check` and `cargo check` |
| `just lint` | Biome, file naming (`ls-lint`) and clippy |
| `just deny` | `cargo deny check` |
| `just format` / `just format-check` | Apply / verify formatting on both boundaries |
| `just check` | `format-check lint typecheck test deny`, non-mutating |

## Where things go

| Kind | Location | Naming |
|---|---|---|
| Frontend feature code | `src/features/<feature>/<layer>/` (`api`, `ui`, `state`) | kebab-case |
| Feature public entry point | `src/features/<feature>/index.ts` | `index.ts` |
| Frontend → backend calls | `src/features/<feature>/api/` wrappers around `invoke`; never call `invoke` from components | kebab-case |
| App composition root | `src/app/` | kebab-case |
| Code shared by 2+ features | `src/shared/<concept>/` (created when a second feature needs it) | kebab-case |
| Frontend tests | `tests/` mirroring `src/` | `<source-name>.test.ts` |
| Domain records and rules | `src-tauri/crates/ontara-core/src/`; one file per table, in `being/`, `place/`, `polity/`, `faction/`, `belief/`, `item/`, `event/` | snake_case, named after the table |
| Persistence, project lifecycle, migrations | `src-tauri/crates/ontara-storage/src/`, SQL in `migrations/` | snake_case modules, `NNNN_<name>.sql` |
| Tauri commands (adapters only) | `src-tauri/src/<feature>/commands.rs`, registered in `src-tauri/src/lib.rs` | snake_case |
| Permissions | `src-tauri/capabilities/*.json` | |
| App configuration | `src-tauri/tauri.conf.json` | |
| Rust unit tests | in-file `#[cfg(test)] mod tests` | |
| Rust integration tests | `<crate>/tests/` | one file per feature |
| Scripts | `scripts/` | kebab-case |
| Docs, ADRs, glossary | `docs/`, `docs/decisions/`, `docs/glossary.md` | kebab-case |
| Community files (README, CONTRIBUTING, CODE_OF_CONDUCT, SECURITY, SUPPORT, CHANGELOG, LICENSE) | repo root | upper-case |
| CODEOWNERS, PULL_REQUEST_TEMPLATE, issue templates, Dependabot | `.github/` | |
| Git hooks | `.husky/` | |
| CI | `.github/workflows/` | |

Root files outside the usual manifests and tool configs: `index.html` is the Vite entry (Vite resolves it from the project root).

## Rules

- Domain rules belong in `ontara-core`, never in the frontend and never in Tauri commands. Commands validate input, call the domain and map results to DTOs.
- Names are English. Reuse the terms in `docs/glossary.md`; add a term in the same change that introduces it.
- Look at the nearest sibling file before creating one. Do not create `utils/`, `helpers/` or `common/` folders.
- Conventional Commits are enforced locally and in CI.
- Do not add Tauri permissions without justifying each one in the PR.
- Every `allow` of a lint carries a comment with the reason.
