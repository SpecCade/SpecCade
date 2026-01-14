# speccade (Rust)

Deterministic asset pipeline: takes a JSON `Spec` and produces artifacts (WAV/PNG/XM/IT/…) plus a report.

## Start Here (Canonical)

- Repo overview + how to run: `README.md`
- Architecture + determinism model: `ARCHITECTURE.md`
- Determinism expectations by backend: `PARITY_MATRIX.md`

## Repo Map (High Level)

- `crates/speccade-spec/` — core spec types, validation, hashing (source of truth)
- `crates/speccade-cli/` — `speccade` CLI (validate/generate/format/migrate)
- `crates/speccade-backend-*/` — generation backends (Tier 1: Rust-only; Tier 2: external tools like Blender)
- `crates/speccade-tests/` — integration + determinism validation
- `schemas/` — JSON schemas for the spec format
- `golden/` — golden outputs used by tests
- `packs/` — example packs/inputs

## Quick Commands

- Tests: `cargo test`
- Determinism/integration tests: `cargo test -p speccade-tests`
- CLI help: `cargo run -p speccade-cli -- --help`

## Determinism Guardrails (Tier 1 backends)

- Given the same validated spec + seed, output must be **byte-identical** across runs.
- Avoid non-deterministic inputs: wall clock time, OS RNG, thread timing, filesystem ordering.
- Prefer stable iteration order (`BTreeMap`/sorted keys) and explicit rounding/quantization where needed.

## When Changing the Spec Format

- Update `crates/speccade-spec/` (types + validation) and `schemas/` together.
- Update/extend golden tests in `crates/speccade-tests/` and/or `golden/`.
- Keep `PARITY_MATRIX.md` accurate if behavior differs across backends.

