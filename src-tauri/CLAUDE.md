# Backend (`src-tauri/`)

Rust 2024, Cargo workspace. The root package `ontara` (lib `app_lib`) is the Tauri adapter. `crates/ontara-core` is the domain engine and `crates/ontara-storage` is persistence.

## Rules

- `ontara-core` must not depend on Tauri, the filesystem, a database or any external provider. It defines the repository contracts (`ModelReader`, `ModelRepository`) and is tested without Tauri.
- `ontara-storage` implements those contracts over SQLite and owns the portable project folder. It contains no domain rules: every write goes through `ontara_core::ModelEditor`, reached with `Project::edit`.
- Schema changes are new files in `crates/ontara-storage/migrations/` (`NNNN_<name>.sql`) registered in `migration.rs`. A shipped migration is never edited.
- Time is injected: domain and storage operations take a `Timestamp` instead of reading the clock.
- One file per record type, named after its table (`being_trait.rs`), grouped in an area folder (`being/`, `place/`, `polity/`). Every persisted record type implements `Record`, `Reader<R>` and `Writer<R>`. A new record type means: the table in a migration, the type and its `validate`, its identifier, its `Reader`/`Writer` in `ontara-storage/src/sqlite_model/<area>.rs`, its editor operations in `model_editor/<area>.rs`, and tests at both levels.
- Editor operations are named `add_`, `update_` and `remove_` for records with their own identifier, and `set_` and `remove_` for records keyed by an entity or a pair of entities.
- Removal is driven by the schema's foreign keys: a required reference is removed with what it points at, an optional one is cleared. A new table is covered without code, but `OWNED_BY_ENTITY` in `sqlite_model/removal.rs` must list it if its rows are part of the entity rather than references to it.
- Every write goes through `Workspace::record("<feature>.<operation>", …)`, which names the edit in the undo history; the label is a stable machine name the interface words. Undo is a journal installed by `journal::install` from the schema, so a new table needs nothing; a migration drops the history. A new source of searchable text needs a trigger in a migration that refreshes `search_document`.
- Dossier sections (the records that describe an entity) are one row each in `src/dossier/sections.rs`: record type, the fields that tie it to entities and its editor operations.
- Assistants never write to the model: a provider answer or an MCP call becomes a `Proposal`, approved change by change through `ModelEditor`. A new operation is a variant of `ProposedChange` (closed set), its application in `proposal.rs` and its text in the interface. Provider calls go through `assistant::Transport`, keys through `SecretStore`, and anything that leaves the computer is gated by the project's `AiPolicy`.
- Behavior that depends on what a type is, never on its name: use `EntityRole`, not a key comparison.
- Tauri commands are adapters: validate arguments, call `ontara-core`, map to a DTO, return. A DTO mirrors a type in `src/features/<feature>/api/` and says so in its doc comment.
- Layout: `src/lib.rs` exposes `run()` and registers commands, `src/main.rs` only calls `run()`. Features are `src/<feature>.rs` plus `src/<feature>/commands.rs`; no `mod.rs`.
- Files and modules are `snake_case`; crate folders under `crates/` are `kebab-case`.
- Lints are inherited from the workspace: `unsafe_code = forbid`, clippy `all = deny`, `pedantic = warn`, `unwrap_used`/`expect_used = warn`, run with `-D warnings`. Every `allow` carries a reason.
- `rust-toolchain.toml` pins the exact toolchain; `rust-version` in `Cargo.toml` is the MSRV and must not exceed it. `Cargo.lock` is committed.
- Dependencies are declared once in `[workspace.dependencies]` and inherited with `.workspace = true`.
- `deny.toml` is audited by `just deny`. Every ignored advisory and every added license names the dependency that needs it.

## Tauri

- `capabilities/default.json` starts from `core:default`; `dialog:allow-open` is added for the native folder and file pickers (decision 0005). Add permissions one at a time, scoped to the narrowest path, each justified in the PR.
- The CSP in `tauri.conf.json` is strict. Do not add `unsafe-eval` or `unsafe-inline` for scripts; extend `connect-src` only with origins the app really calls.
- The bundle identifier `com.sxnnyside.ontara` never changes after the first release.
