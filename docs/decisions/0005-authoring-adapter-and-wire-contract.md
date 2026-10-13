# 0005. Authoring adapter and wire contract

Status: accepted

## Context

Authoring a universe needs the frontend to browse, create, relate, structure and attach, with the
domain still the only place rules live. The adapter must stay thin and the contract between
Rust and TypeScript must be easy to mirror and test.

## Decision

**Domain records cross the boundary as they are.** They serialize with their own field names
(snake_case) and tagged enums (`Temporal` by `kind`, property content by `type`, catalog entries
by `catalog`, connections by `via`). The adapter defines a DTO only for a read model that joins
records, such as a relationship with both ends resolved.

**Adapter logic is plain functions over the open project.** Each feature is `src/<feature>.rs`
with functions taking `&Workspace`, plus a thin `commands.rs` registered in `lib.rs`. The
functions are tested against a real temporary project; commands only unwrap Tauri state.

**One open project, one lock.** `Workspace` holds `Mutex<Option<Project>>` and exposes `read`,
`write`, `replace` and `close`. Failures cross as `CommandError { code, message }`; the code is
what the frontend branches on.

**Configuration is data.** Types, relationship types, canon states, editorial levels,
appearance kinds and vocabularies are editable catalog rows. Their keys never change; structural
fields that existing records depend on are immutable, and an entry that is built in or in use
cannot be removed.

**Assets are imported, not linked.** An imported file is copied into `assets/` under a safe name,
deduplicated by SHA-256, and previewed through the backend with a size cap.

**Interface texts are localized by area.** Texts live in groups, one file per area of the
interface, and each key holds all its translations, so a missing language is a type error.
Built-in catalog entries (types, relationship types, canon states, properties, vocabulary terms)
are stored with their English default and shown in the interface language while that default is
unchanged; a label the author renamed is shown as written, and an unchanged label is stored back
as the default so switching language keeps translating it.

## Permissions

`dialog:allow-open` is the only permission added to `core:default`: choosing a project folder or
a file to import needs the native picker, and the backend reads only the path the author chose.

## Consequences

- A new command needs: the core operation and its tests, an adapter function with a test, a
  `commands.rs` wrapper, a registration, a typed wrapper in the feature's `api/` and a test of it.
- Mirrors are kept by hand; each DTO names its TypeScript counterpart and vice versa.
