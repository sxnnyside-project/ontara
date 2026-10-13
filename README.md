# Ontara

![Banner](ontara-banner.png)

![Version](https://img.shields.io/badge/version-0.1.0-blue)
![License](https://img.shields.io/badge/License-MIT-green)
![MSRV](https://img.shields.io/badge/MSRV-1.90-orange)
[![CI](https://github.com/sxnnyside-project/ontara/workflows/CI/badge.svg)](https://github.com/sxnnyside-project/ontara/actions)

<p align="center">
  <strong>Local-first ✦ Typed Relationship Graph ✦ Narrative Consistency</strong><br>
  <em>Desktop environment for modeling and maintaining complex fictional universes.</em>
</p>

<p align="center">
  <a href="#about">About</a> ✦
  <a href="#features">Features</a> ✦
  <a href="#installation">Installation</a> ✦
  <a href="#usage">Usage</a> ✦
  <a href="#architecture">Architecture</a> ✦
  <a href="#contributing">Contributing</a>
</p>

---

## About

**Ontara** is a desktop application for building, organizing and exploring fictional universes as a structured narrative model of entities, typed relationships, events, appearances and editorial structure.

General-purpose notes tools store a universe as documents, so contradictions, indirect relationships and appearances across chapters are left to manual upkeep. Ontara keeps one queryable model and derives every view from it.

Projects are portable local folders you own. Nothing requires an account or a network connection. The project is in its foundation phase: the application shell, tooling and architecture exist, and the product features are not implemented yet.

### Philosophy

> *"The system should understand the author's world without taking ownership of the author's creativity."*

Developed by Sxnnyside Project in collaboration with Crown Red Editorial (Sxnnyside Project x Crown Red Editorial).

## Features

The features below define the product scope. They are not implemented yet.

- **Typed Model**: Configurable entity types, directed and undirected relationships, canon status and custom terminology.
- **Graph Engine**: Rust domain engine for traversal, filtering and neighborhood queries over the narrative graph.
- **Editorial Structure**: Series, volumes, parts, chapters and scenes, independent of in-universe chronology.
- **Appearances and Events**: Classified appearances per chapter or scene, and events ordered by in-universe time.
- **Specialized Views**: Table, tree, graph, appearance matrix and timeline over the same data.
- **Consistency Checks**: Deterministic, explainable findings that never modify canon on their own.
- **Portable Projects**: SQLite persistence, relative asset references, export, backup and recovery.
- **Optional AI**: Bring-your-own-key generation and a local MCP server, always behind explicit approval.

## Installation

### Prerequisites

- [Bun](https://bun.sh) (1.4 or newer)
- [Rust](https://rustup.rs) (the exact toolchain is pinned in `src-tauri/rust-toolchain.toml`)
- [just](https://just.systems) and [cargo-deny](https://github.com/EmbarkStudios/cargo-deny)
- Tauri's [platform prerequisites](https://v2.tauri.app/start/prerequisites/) for your operating system

### From Source

```bash
git clone https://github.com/sxnnyside-project/ontara.git
cd ontara

just install
```

## Usage

```bash
just dev          # run the desktop app with hot reload
just check        # the full quality gate (also run by CI and the pre-commit hook)
just test         # frontend and Rust test suites
just build        # compile the frontend and the release binary
```

`just --list` shows every recipe. Commits follow [Conventional Commits](https://www.conventionalcommits.org); a local hook enforces them.

## Architecture

```
ontara/
├── src/          # Frontend application (TypeScript + Svelte 5, Vite)
├── src-tauri/    # Rust workspace: Tauri adapter and the ontara-core domain engine
├── tests/        # Frontend tests, mirroring src/
├── scripts/      # Git workflow scripts
└── docs/         # Architecture, glossary and decision records
```

For conventions and where files go, see [CLAUDE.md](CLAUDE.md) and the one inside each boundary. For the layers and the frontend/backend contract, see [docs/architecture.md](docs/architecture.md).

## Contributing

Contributions are accepted. See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

Before contributing, read the [Code of Conduct](CODE_OF_CONDUCT.md).

## License

This project is licensed under the MIT License — see the [LICENSE](LICENSE) file for details.

---

<p align="center">
  <strong>Ontara</strong> — Sxnnyside Project x Crown Red Editorial<br>
  <em>&copy; 2026 Sxnnyside Project</em>
</p>
