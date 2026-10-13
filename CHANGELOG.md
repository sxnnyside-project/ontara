# Changelog

All notable changes to **Ontara** are documented here.

This project follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/)
and [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [Unreleased]

### Added

- Application shell: Tauri v2, Svelte 5 and Bun frontend, Rust workspace with the `ontara-core` domain engine crate.
- Quality gate (`just check`), git hooks, CI and dependency updates.
- Relational narrative model: 79 tables covering beings (profile, appearance over time, traits, marks, languages, family, partnerships), places (containment by kind and period, borders, census), polities (offices and tenures, citizenship, diplomacy, succession), factions, culture and belief, items, events and causality, all with periods and canon states.
- Entity type hierarchy with inherited traits (a city is a settlement is a place) and 85 starting traits such as hair color, eye color, fears and quirks.
- Configurable canon states, vocabularies and calendars (months, leap rules, eras) on a shared timeline; a moment is what the author tells, with an optional day.
- Rule-enforcing `ModelEditor`: typed references honoring subtypes, loop-free hierarchies, explicit resolution when removing referenced entities.
- Portable project folders in `ontara-storage`: manifest, SQLite schema, versioned migrations, verified backups and restore, integrity and missing-asset checks.
- Authoring experience: create and edit entities with names, tags, notes, traits and canon history; configure entity types, relationship types, canon states, editorial levels, properties and vocabularies; typed relationships with context and period; editorial structure with ordering; asset import, attachment and preview; quick find and back/forward navigation across links.
- Specialized views: relationship graph exploration, appearance matrix by section or event, in-universe timeline with narrative-order comparison and event editing, birth and death dates for beings.
- Deterministic consistency checks (14 rules, CC-01 to CC-14) with findings that open the entity involved and a live error count in the header.
- Interface rebuilt on one layout: an activity rail for views, a navigator with entities and structure, a shared view frame with toolbar and detail panel, a universe home with onboarding, and an add-entity dialog that asks only for a name and a type.
- Interface texts organized by area with every translation beside its key; built-in types, relationship types, canon states, properties and vocabulary terms are shown in the interface language until renamed.
- Undo and redo of every edit, kept by the database so each new kind of record is covered without code, with a history dialog, keyboard shortcuts and a named edit in the status bar; history survives closing the project.
- Full-text search across names, tags, summaries, descriptions, notes and property text, ignoring case and accents, with the matching passage shown; the command palette (Ctrl or Cmd with K) searches the universe and runs commands.
- Entity dossier: profile, traits, marks, languages, family ties, partnerships, citizenship and offices of beings; and the profiles and records of deities, species, places, polities, factions, languages, religions, cultures and items, each editable in one dialog.
- Daily automatic backups kept to the latest seven, a backups and export dialog with restore, and export of the universe as a Markdown document or a complete JSON copy.
- A small example universe with one deliberate contradiction, a getting-started list on the universe home, and Ctrl or Cmd with Enter to confirm any dialog.
- Accessibility: contrast checked in both themes, visible keyboard focus, reduced motion respected, named controls and dialogs; an automated check runs with the linter.
- Assistants: configure providers with your own keys (kept by the operating system), ask for descriptions, relationships, analysis or new entities, and review each suggestion before anything changes; what an assistant creates is provisional and marked as such.
- A project decides what assistants may reach: sending text to providers over the internet and MCP access both start off, and every request shows exactly what it would send.
- Local MCP server (`ontara-mcp`) that lets an assistant on this computer read a project and propose changes, with no network port.
- Security and privacy tests for the assistant surface, and `docs/security-privacy.md` mapping each threat to the test that holds it.

### Fixed

- Graph: the legend's Customize button and the zoom buttons did nothing, because the canvas captured the pointer before the click reached them.
