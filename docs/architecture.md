# Architecture

Ontara is a desktop application with a Svelte frontend, a Rust domain engine and a Tauri adapter between them. This document describes the structure that exists today; the product scope lives in the product requirements.

## Topology

One repository, two language boundaries, one task runner (`Justfile`).

```text
ontara/
├── src/                      Frontend (Svelte 5 + TypeScript)
│   ├── app/                  Composition root: bootstrap and shell
│   └── features/<feature>/   One domain concept per folder: api/, ui/, state/
│   └── (project, catalog, entity, relationship, editorial, asset, navigation, graph, matrix, timeline, consistency)
├── tests/                    Frontend tests, mirroring src/
├── src-tauri/                Rust workspace
│   ├── src/                  Tauri adapter: commands only
│   ├── crates/ontara-core/   Domain engine: model, rules, repository contracts
│   ├── crates/ontara-storage/ Persistence: project folders, SQLite, migrations, backups
│   ├── capabilities/         Tauri permissions
│   └── tauri.conf.json
├── docs/                     Architecture, glossary, decisions
├── scripts/                  Repository scripts (commit validation)
└── .github/                  CI, Dependabot, community files
```

## Layers and dependency direction

```text
Svelte UI ──▶ features/<feature>/api ──invoke──▶ Tauri commands ──▶ ontara-core
```

- **UI** renders state and collects intent. It owns no domain rules.
- **`api/` wrappers** are the only place that calls `invoke`. They give each command a typed signature.
- **Tauri commands** are adapters: validate input, call the domain, map to a DTO.
- **`ontara-core`** holds authoritative rules: model, graph operations, validation. It has no dependency on Tauri or the UI.

`ontara-storage` implements the domain's repository contracts over SQLite and owns the portable project folder. Graph traversal, consistency checks, AI providers and the MCP server will each arrive as their own module or crate behind the same direction: adapters depend on the domain, never the reverse.

## Domain model

| Area | Records | Where |
|---|---|---|
| Entities | entity, names, tags, notes; types with a hierarchy and roles | `ontara-core` flat modules |
| Beings | profile, appearance over time, marks, traits, languages, family ties, partnerships; deity and species profiles | `ontara-core/src/being/` |
| Places | profile, containment by kind and period, borders, resources, census | `ontara-core/src/place/` |
| Polities | profile, languages, symbols, succession, offices and tenures, citizenship, diplomacy | `ontara-core/src/polity/` |
| Factions | profile, parent factions, memberships | `ontara-core/src/faction/` |
| Culture and belief | practices, languages, religions, tenets, holy places, adherence | `ontara-core/src/belief/` |
| Items | profile, powers, ownership over time | `ontara-core/src/item/` |
| Events | detail, participants, causality | `event_detail.rs`, `ontara-core/src/event/` |
| Narrative | appearances, editorial nodes | flat modules |
| Generic | relationships, typed properties and their values | flat modules |
| Time | calendars, moments, periods | `calendar.rs`, `temporal.rs` |
| Rules | `ModelEditor`, the only sanctioned way to write | `model_editor/` |
| Persistence | `Project`: create, open, edit (transactional), backup, restore, integrity | `ontara-storage` |

Every record type is one file named after its table. Decisions:
[0003](decisions/0003-project-format-and-persistence.md) (format and persistence) and
[0004](decisions/0004-narrative-domain-model.md) (model). The relational catalog is in
[data-model/overview.md](data-model/overview.md).

## Project folder

```text
<Name>.ontara/
├── ontara.project.json   Manifest: format version, project id, name
├── data/ontara.sqlite    The model; schema version in PRAGMA user_version
├── assets/               User files, referenced by relative path
├── exports/
└── backups/              Verified database backups
```

The format, migration and recovery policy is in [decision 0003](decisions/0003-project-format-and-persistence.md).

## Frontend / backend contract

Domain records cross the boundary as they are; an adapter DTO exists only for a read model that joins records. Each command has a typed wrapper in the matching `api/` folder, and the Rust DTO's doc comment names the TypeScript file and vice versa. Adapter logic is plain functions over `Workspace` (one open project behind a lock), tested on a real temporary project, with a thin `commands.rs` per feature. See [0005](decisions/0005-authoring-adapter-and-wire-contract.md).

## Analysis

Graph exploration, the appearance matrix, the timeline and the consistency checks are read-only domain code in `ontara-core` with pure cores that run on hand-made records. The frontend views share one focus entity through the navigation state. See [0006](decisions/0006-analysis-views-and-consistency-checks.md).

## History and search

Every edit is a change set kept by the database as it happens, so any edit can be undone and redone and each new table is covered without code; full-text search is an index the migrations own and keep current. See [0007](decisions/0007-change-history-and-full-text-search.md).

## Workflow

The dossier of an entity (the records that describe it), the command palette, backups and export, the example universe and the getting-started list are what make the workflow quick and forgiving; accessibility, scale and the primary path are checked automatically, and usability against a person's current process by [the usability protocol](usability-protocol.md). See [0008](decisions/0008-dossier-and-the-lovable-workflow.md).

## Assistants

Providers and MCP clients only propose: answers become proposals that wait in their own tables until the author approves each change, and approving goes through the model editor as one undoable edit. Sending text to a provider over the internet and MCP access are off until a project turns them on; keys live in the system keychain. See [0009](decisions/0009-assistants-proposals-and-mcp.md) and [security and privacy](security-privacy.md).

## Security baseline

- Strict CSP (`script-src 'self'`, no `unsafe-eval`).
- Capabilities start from `core:default` and grow one justified permission at a time.
- No network access is required by any core feature. The only outbound calls are to a provider the author configured and allowed, made by the backend, on request.

## Decisions

Recorded in `docs/decisions/`.
