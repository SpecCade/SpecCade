# Contributing to SpecCade

This guide covers development setup, repo structure, and how to add/modify backends while keeping the spec contract consistent.

## Development Setup

### Prerequisites

- **Rust:** recent stable toolchain (must support the workspace `Cargo.lock` format)
- **Blender:** 3.6+ (only required for Blender-backed backends)
- **Python:** 3.x (optional; only required for `speccade migrate --allow-exec-specs`)

### Build

```bash
# From the SpecCade repo root
cargo build
```

### Install CLI locally

```bash
cargo install --path crates/speccade-cli
speccade --version
```

## Repo Structure

See [`ARCHITECTURE.md`](../ARCHITECTURE.md) for the full crate map and repo layout.

## Running Tests

```bash
# Unit tests across the workspace
cargo test --workspace

# Integration tests (CLI + format validators)
cargo test -p speccade-tests
```

## Formatting and Linting

```bash
cargo fmt --all
cargo clippy --workspace --all-targets
```

## Adding or Changing a Recipe Kind

SpecCade treats the Rust types in `speccade-spec` + the CLI validation rules as the SSOT for the public contract.

When adding a new recipe kind or changing an existing one:

1. Update `crates/speccade-spec/src/recipe/mod.rs`:
   - Add a `RecipeKind` variant (string form)
   - Add a `Recipe::parse_kind` match arm (if applicable)
   - Add a typed `Recipe::as_*` helper if you want validation/dispatch to parse params
2. Define the params type under the appropriate module (e.g. `crates/speccade-spec/src/recipe/audio/`).
   - Prefer `#[serde(deny_unknown_fields)]` for strictness.
3. Update validation in `crates/speccade-spec/src/validation/mod.rs`:
   - Parse params for the recipe kind (return `E012` on failure)
   - Enforce output constraints that match the backend behavior (formats, counts, required output kinds)
4. Implement/extend the backend crate.
5. Wire the backend into `crates/speccade-cli/src/dispatch/mod.rs` (and any relevant module under `crates/speccade-cli/src/dispatch/`).
6. Update docs in `docs/spec-reference/` and any relevant RFCs.

## Golden Specs

Reference specs live under:

```
golden/speccade/specs/
```

These are useful for manual testing and as examples when updating docs.
