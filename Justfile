set shell := ["bash", "-euo", "pipefail", "-c"]

# List every recipe.
default:
    @just --list

# Bootstrap both toolchains and install the git hooks.
install:
    bun install --frozen-lockfile
    cd src-tauri && cargo fetch --locked
    command -v cargo-deny >/dev/null || cargo install --locked cargo-deny

# Run the desktop app with hot reload.
dev:
    bun run tauri dev

# Compile the frontend and the release binary without bundling installers.
build:
    bun run tauri build --no-bundle

# Run the frontend and Rust test suites.
test:
    bun run test
    cd src-tauri && cargo test --workspace --locked

# Type-check the frontend and the Rust workspace.
typecheck:
    bun run typecheck
    cd src-tauri && cargo check --workspace --all-targets --locked

# Static analysis for both boundaries, including file naming.
lint:
    bun run lint
    cd src-tauri && cargo clippy --workspace --all-targets --locked -- -D warnings

# Audit Rust dependencies (advisories, licenses, bans, sources).
deny:
    command -v cargo-deny >/dev/null || { echo "cargo-deny is missing: run 'just install'" >&2; exit 1; }
    cd src-tauri && cargo deny check --hide-inclusion-graph

# Apply formatting to both boundaries.
format:
    bun run format
    cd src-tauri && cargo fmt --all

# Verify formatting without modifying files.
format-check:
    bun run format:check
    cd src-tauri && cargo fmt --all --check

# Full quality gate. Non-mutating; CI and the pre-commit hook call this.
check: format-check lint typecheck test deny

# Remove build output and caches.
clean:
    bun run clean
    cd src-tauri && cargo clean
