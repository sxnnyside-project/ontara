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
