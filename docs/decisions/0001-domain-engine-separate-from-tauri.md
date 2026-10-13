# 0001. Domain engine as a separate crate from the Tauri adapter

Status: accepted

## Context

Domain rules (graph traversal, narrative validation, project integrity) must be testable without Tauri and reusable by more than one entry point: the desktop UI now, a local MCP server later. Tauri commands that embed rules would tie both to the webview runtime.

## Decision

The Cargo workspace root is `src-tauri/`. The root package is the Tauri adapter. Domain logic lives in `src-tauri/crates/ontara-core`, which does not depend on Tauri. Commands validate input, call the domain and map results to DTOs.

## Consequences

- Domain tests run with plain `cargo test`, with no webview.
- Adding an adapter (MCP, CLI) means a new consumer of `ontara-core`, not a change to it.
- Every command needs a DTO mapping layer, which is a small, deliberate cost.
