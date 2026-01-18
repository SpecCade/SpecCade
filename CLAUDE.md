# SpecCade - Claude Code Instructions

## Purpose

SpecCade is a deterministic asset pipeline. It validates a `Spec` (JSON or Starlark) and generates assets via backends.

## Start Here

- `README.md` — usage and examples
- `docs/README.md` — docs map / what to read first
- `ARCHITECTURE.md` — crate map + determinism model
- `PARITY_MATRIX.md` — what “deterministic” means per backend/tier

## Ground Rules (High Signal)

- Treat `crates/speccade-spec/` as the source of truth for the spec format (types + validation + hashing).
- Tier 1 backends (Rust-only) should be byte-identical for the same spec hash/seed.
- Tier 2 backends (external tools like Blender) should have explicit validation rules (don't claim byte-identical output unless enforced).

## Key Files for Starlark Development

**Compiler pipeline:**
- `crates/speccade-cli/src/input.rs` - Unified spec loading, dispatches by extension (.json/.star)
- `crates/speccade-cli/src/compiler/mod.rs` - Starlark compiler entry point, config, timeout
- `crates/speccade-cli/src/compiler/eval.rs` - Starlark evaluation with safety limits
- `crates/speccade-cli/src/compiler/convert.rs` - Starlark Value to JSON conversion
- `crates/speccade-cli/src/compiler/stdlib/mod.rs` - Stdlib registration

**Stdlib modules:**
- `crates/speccade-cli/src/compiler/stdlib/core.rs` - spec(), output() scaffolding
- `crates/speccade-cli/src/compiler/stdlib/audio/mod.rs` - Audio synthesis helpers
- `crates/speccade-cli/src/compiler/stdlib/audio/synthesis/mod.rs` - Synth-specific functions
- `crates/speccade-cli/src/compiler/stdlib/music/mod.rs` - Tracker composition helpers
- `crates/speccade-cli/src/compiler/stdlib/music/instruments.rs` - Instrument definitions
- `crates/speccade-cli/src/compiler/stdlib/music/patterns.rs` - Pattern composition
- `crates/speccade-cli/src/compiler/stdlib/texture/mod.rs` - Texture node graph helpers
- `crates/speccade-cli/src/compiler/stdlib/mesh.rs` - Mesh primitive helpers

**Budget enforcement:**
- `crates/speccade-spec/src/validation/budgets.rs` - BudgetProfile, profiles, limits

**Documentation:**
- `docs/starlark-authoring.md` - Starlark authoring guide
- `docs/stdlib-reference.md` - Stdlib function reference
- `docs/budgets.md` - Budget system documentation

**Tests and examples:**
- `crates/speccade-tests/tests/starlark_input.rs` - Starlark input loading tests
- `golden/starlark/` - Golden Starlark specs for integration tests

## Common Workflows

### Spec authoring and validation

- **Author specs in Starlark:** Use `.star` files for ergonomic authoring with stdlib helpers. See `docs/starlark-authoring.md` and `docs/stdlib-reference.md`.
- **Evaluate Starlark to JSON IR:** `cargo run -p speccade-cli -- eval --spec file.star --pretty` to preview canonical IR.
- **Validate with budget profiles:** `cargo run -p speccade-cli -- validate --spec file.star --budget strict` to enforce resource limits.
- **Generate with budgets:** `cargo run -p speccade-cli -- generate --spec file.star --out-root ./out --budget strict`

### Development workflows

- **Add/change spec fields:** update `speccade-spec` + `schemas/` + `speccade-tests` (and `golden/` if needed).
- **Add a new backend feature:** keep the spec change minimal, implement in the backend crate, then add determinism/integration coverage.
- **Add a new stdlib function:**
  1. Add function to appropriate module in `crates/speccade-cli/src/compiler/stdlib/` (audio, texture, mesh, music, or core)
  2. Register in module's `register()` function
  3. Add tests using `eval_to_json()` helper
  4. Document in `docs/stdlib-reference.md`
- **Debug a determinism regression:** start from the failing golden/test case in `crates/speccade-tests/` and trace inputs/seed hashing in `speccade-spec`.
- **Adjust budget limits:** modify `crates/speccade-spec/src/validation/budgets.rs` profiles or add new profiles.

## Quick Commands

### Testing
- `cargo test`
- `cargo test -p speccade-tests`
- `cargo test -p speccade-cli` (includes Starlark compiler tests)

### CLI
- `cargo run -p speccade-cli -- --help`
- `cargo run -p speccade-cli -- eval --spec file.star --pretty` - compile Starlark to JSON IR
- `cargo run -p speccade-cli -- validate --spec file.star --budget strict` - validate with budget
- `cargo run -p speccade-cli -- generate --spec file.star --out-root ./out --budget strict` - generate with budget
