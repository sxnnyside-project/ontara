# 0003. Project format and persistence

Status: accepted (resolves open decisions 1 and 10 of the product definition for the foundation phase)

## Context

A project must be a portable folder, survive interrupted writes, refuse versions it does not understand, and never depend on machine-specific paths. Domain rules must not be coupled to a database.

## Decision

**Folder layout.** A project is a folder with `ontara.project.json` at its root and the sub-folders `data/`, `assets/`, `exports/` and `backups/`. Only layout relative to the root is ever stored.

**Manifest.** `ontara.project.json` holds `format_version`, `project_id`, `name`, `created_at` and `created_with`. It is written to a temporary file and renamed into place. It is written last when a project is created, so a folder without a manifest is never taken for a project. `format_version` covers the manifest and folder layout only.

**Database.** One SQLite file, `data/ontara.sqlite`, in WAL mode with foreign keys enforced. The schema version lives in the database (`PRAGMA user_version`), not in the manifest, so the two cannot disagree. When the last connection closes the log is checkpointed and removed: a closed project's database is a single file, safe to move.

**Migrations.** Ordered, append-only SQL files in `crates/ontara-storage/migrations/`, each applied in its own transaction. A shipped migration is never edited. Opening a project:

| Found | Result |
|---|---|
| format or schema newer than this build | refused, nothing rewritten, message says to update Ontara |
| schema older | refused with `MigrationRequired` unless the caller chooses to migrate; migrating takes a verified backup first |
| integrity check fails | refused with guidance to restore the latest backup |

**Backups.** Written through SQLite's online backup API under a temporary name, verified with a full integrity check, then given their final name, so any file in `backups/` has passed verification. Restoring verifies the backup first and keeps the state it replaces as `pre-restore-<timestamp>` in `backups/`, so a restore is itself reversible.

**Identity.** Entities and relationships are identified by UUID v7 (time-ordered). Entity and relationship types are identified by a stable snake_case `TypeKey`; labels are separate and customizable, so renaming a label never touches identity.

**Repository contract.** `ontara-core` defines `Reader<R>` and `Writer<R>` per record type plus `ModelQueries` for the questions the rules ask. `ontara-storage` implements them over SQLite. Writes go through `ModelEditor`, which enforces the domain rules, and `Project::edit` runs each edit in one transaction.

**Schema conventions.** The full model is described in `docs/data-model/overview.md`. In short:
every table is `STRICT`; catalogs and vocabularies are tables so every use is a foreign key; a
fact that changes over time is a table with a period over `instant` rows, never an overwritten
value; anything an author states carries canon state, origin and timestamps. There are no JSON
columns: structure is columns and child tables. Polymorphic references (citations, canon
history) name their table and are checked by the integrity report, since a foreign key cannot
express them.

**Version 1 is not frozen.** `0001_initial_schema.sql` is the schema of the foundation phase and was rewritten before any release. From the first release on, a shipped migration is never edited.

## Consequences

- Moving, copying or backing up a closed project is a folder operation.
- A failed multi-record edit leaves nothing behind; a crash leaves the last committed state.
- Seeding built-in catalog rows happens at creation; a build that adds built-ins later must do it through a migration.
- Two processes opening the same project are not coordinated beyond SQLite's own locking.
