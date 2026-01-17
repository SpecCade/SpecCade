# speccade (Rust)

Deterministic asset pipeline: takes a `Spec` (JSON or Starlark) and produces artifacts (WAV/PNG/XM/IT/GLB/...) plus a report.

## Start Here (Canonical)

- Repo overview + how to run: `README.md`
- Docs map (what to read first): `docs/README.md`
- Architecture + determinism model: `ARCHITECTURE.md`
- Determinism expectations by backend: `PARITY_MATRIX.md`

## Repo Map (High Level)

- `crates/speccade-spec/` — core spec types, validation, hashing, budgets (source of truth)
- `crates/speccade-cli/` — `speccade` CLI (eval/validate/generate/format/migrate) + Starlark compiler
- `crates/speccade-cli/src/compiler/` — Starlark-to-JSON compiler pipeline
- `crates/speccade-cli/src/compiler/stdlib/` — Starlark stdlib (audio, texture, mesh, music, core)
- `crates/speccade-backend-*/` — generation backends (Tier 1: Rust-only; Tier 2: external tools like Blender)
- `crates/speccade-tests/` — integration + determinism validation
- `schemas/` — JSON schemas for the spec format
- `golden/` — golden outputs used by tests
- `golden/starlark/` — golden Starlark specs for integration tests
- `packs/` — example packs/inputs
- `docs/` — documentation (`docs/README.md`, starlark authoring, stdlib reference, budgets, spec reference)

## Quick Commands

- Tests: `cargo test`
- Determinism/integration tests: `cargo test -p speccade-tests`
- Starlark compiler tests: `cargo test -p speccade-cli`
- CLI help: `cargo run -p speccade-cli -- --help`
- Eval Starlark to JSON: `cargo run -p speccade-cli -- eval --spec file.star --pretty`
- Validate with budget: `cargo run -p speccade-cli -- validate --spec file.star --budget zx-8bit`
- Generate with budget: `cargo run -p speccade-cli -- generate --spec file.star --out-root ./out --budget strict`

## Determinism Guardrails (Tier 1 backends)

- Given the same validated spec + seed, output must be **byte-identical** across runs.
- Avoid non-deterministic inputs: wall clock time, OS RNG, thread timing, filesystem ordering.
- Prefer stable iteration order (`BTreeMap`/sorted keys) and explicit rounding/quantization where needed.

## When Changing the Spec Format

- Update `crates/speccade-spec/` (types + validation) and `schemas/` together.
- Update/extend golden tests in `crates/speccade-tests/` and/or `golden/`.
- Keep `PARITY_MATRIX.md` accurate if behavior differs across backends.

## Starlark Authoring

SpecCade supports authoring specs in Starlark (.star files) which compile to canonical JSON IR:

**Pipeline:** `.star file -> compiler -> JSON IR -> validation -> backend`

**Stdlib modules:**
- `core` - spec(), output() scaffolding
- `audio` - envelope(), oscillator(), fm_synth(), filter(), effect(), layer()
- `music` - instrument(), pattern(), song()
- `texture` - noise_node(), gradient_node(), graph()
- `mesh` - mesh_primitive(), mesh_recipe()

**Budget system:**
- Profiles: `default`, `strict`, `zx-8bit`
- Enforced at validation stage before generation
- Use `--budget <profile>` flag with validate/generate commands

**Key files:**
- `crates/speccade-cli/src/input.rs` - spec loading dispatcher
- `crates/speccade-cli/src/compiler/` - Starlark compiler
- `crates/speccade-cli/src/compiler/stdlib/` - stdlib functions
- `crates/speccade-spec/src/validation/budgets.rs` - budget profiles
- `docs/starlark-authoring.md` - authoring guide
- `docs/stdlib-reference.md` - stdlib function reference
- `docs/budgets.md` - budget documentation
