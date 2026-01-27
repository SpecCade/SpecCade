# SpecCade (Rust) - Agent Notes

Deterministic asset pipeline: takes a `Spec` (JSON or Starlark) and produces artifacts (WAV/PNG/XM/IT/GLB/...) plus a report.

## Single Source Of Truth (SSOT)

- Spec contract + validation: `crates/speccade-spec/` and `speccade validate` (`schemas/` is editor assistance, keep in sync)
- CLI commands/flags: `speccade --help` and `speccade <cmd> --help`
- Starlark stdlib surface: `speccade stdlib dump --format json` (code: `crates/speccade-cli/src/compiler/stdlib/`)
- Planned work + decisions: `docs/ROADMAP.md`

## Start Here (Canonical Navigation)

- Repo overview + how to run: `README.md`
- Docs map (what to read first): `docs/README.md`
- Crate map + determinism model: `ARCHITECTURE.md`
- Backend coverage + tier guarantees: `PARITY_MATRIX.md`

## Repo Map (High Level)

- `crates/speccade-spec/` - spec types, validation, hashing, budgets (SSOT for the public contract)
- `crates/speccade-cli/` - `speccade` CLI + Starlark compiler
- `crates/speccade-backend-*/` - generators (Tier 1: Rust-only; Tier 2: external tools like Blender)
- `crates/speccade-lint/` - semantic quality lint rules + analyzers
- `crates/speccade-editor/` and `editor/` - Tauri editor + real-time preview
- `crates/speccade-tests/` - integration + determinism validation
- `schemas/` - JSON schemas (derived; keep aligned with `speccade-spec`)
- `golden/` - golden outputs used by tests
- `packs/` - example packs/inputs
- `docs/` - documentation (start at `docs/README.md`)
- `claude-plugin/` - Claude plugin (agents + references)

## Quick Commands

### Tests

- Workspace: `cargo test --workspace`
- Integration/determinism: `cargo test -p speccade-tests`

### CLI

- Help: `speccade --help`
- Eval Starlark to JSON IR: `speccade eval --spec file.star --pretty`
- Validate (budgets): `speccade validate --spec file.star --budget strict`
- Generate (budgets): `speccade generate --spec file.star --out-root ./out --budget strict`

If `speccade` is not installed, substitute:

- `cargo run -p speccade-cli -- <args>`

## Determinism Guardrails (Tier 1 backends)

- Given the same validated spec + seed, output must be byte-identical across runs.
- Avoid non-deterministic inputs: wall clock time, OS RNG, thread timing, filesystem ordering.
- Prefer stable iteration order (`BTreeMap`/sorted keys) and explicit rounding/quantization where needed.

## When Changing The Public Contract

- Update `crates/speccade-spec/` (types + validation) first.
- Keep `schemas/` and `docs/spec-reference/` aligned with validation.
- Update/extend golden tests in `crates/speccade-tests/` and/or `golden/`.
- Update `PARITY_MATRIX.md` if behavior differs across backends.

## Starlark Authoring

Pipeline: `.star file -> compiler -> JSON IR -> validation -> backend`

Stdlib modules (see `docs/stdlib-reference.md`):

- `core` - spec(), output() scaffolding
- `audio` - synthesis/effects/layers
- `music` - tracker composition helpers
- `texture` - node graphs + specialized recipes
- `mesh` - mesh primitives + modifiers
- `character` - skeletal mesh helpers
- `animation` - skeletal animation helpers

Key code touchpoints:

- `crates/speccade-cli/src/input.rs` - spec loading dispatcher
- `crates/speccade-cli/src/compiler/` - Starlark compiler
- `crates/speccade-cli/src/compiler/stdlib/` - stdlib implementation
- `crates/speccade-spec/src/validation/budgets.rs` - budget profiles
