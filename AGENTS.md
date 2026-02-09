# SpecCade — Agent Notes

Deterministic asset pipeline: Specs (JSON/Starlark) → artifacts (WAV/PNG/XM/IT/GLB) + reports.

## Change Recipes

### Adding a Recipe Kind

This is the highest-coordination task — 7+ files must stay in sync.

1. **Define params** in `crates/speccade-spec/src/recipe/<category>/` with `#[serde(deny_unknown_fields)]`
2. **Add `RecipeKind` variant** in `crates/speccade-spec/src/recipe/mod.rs` — update ALL 6 match blocks:
   - `as_str()`, `parse_kind()`, `asset_type_prefix()`, `is_tier1()`, `as_<recipe>()`, `try_parse_params()`
3. **Add validation** in `crates/speccade-spec/src/validation/` — output constraints in `recipe_outputs*.rs`
4. **Implement backend** in `crates/speccade-backend-*/`
5. **Wire dispatch** in `crates/speccade-cli/src/dispatch/mod.rs` — ALL 3 functions:
   - `dispatch_generate()`, `dispatch_generate_profiled()`, `is_backend_available()`
6. (Tier 2 only) Add `--mode` choice in `blender/speccade/main.py` choices list + handler in `handlers_*.py`
7. **Docs/tests**: spec in `specs/`, update `PARITY_MATRIX.md`

### Adding a Backend Feature

1. Implement in the backend crate (`crates/speccade-backend-*/`)
2. Extend params in `crates/speccade-spec/src/recipe/`
3. Update validation if new constraints apply
4. Add example spec to `specs/`
5. Test: `cargo test -p speccade-tests`

### Adding a Stdlib Function

1. Add function in `crates/speccade-cli/src/compiler/stdlib/<domain>/` using `#[starlark_module]`
2. Register in the domain's `register()` function
3. Add tests in the same module
4. Update stdlib docs (`docs/stdlib-reference.md`)
5. Test: `cargo test -p speccade-cli` then `cargo test -p speccade-tests --test stdlib_snapshot`

### Changing Blender Python

1. Edit handler in `blender/speccade/handlers_*.py`
2. Test: `cargo run -p speccade-cli -- generate --spec <spec> --out-root ./out`
3. For skeletal: verify with `speccade preview-grid --spec <spec>`
4. Key pitfalls:
   - Always use `orient_type='LOCAL'` in `extrude_region_move` for non-vertical bones
   - Call `apply_rotation_only(obj)` AFTER cap removal, not before
   - Bridge edge loops only work for coaxial bones with matching profile segments
   - Always define `shoulder_l`/`shoulder_r` bone meshes to avoid floating arms

## Repo Map

> See `ARCHITECTURE.md` for full crate details, module maps, and dependency graph.

| Directory | Purpose |
|---|---|
| `crates/speccade-spec/` | Spec types, validation, hashing, budgets (SSOT for public contract) |
| `crates/speccade-cli/` | CLI entry point, Starlark compiler, dispatch |
| `crates/speccade-backend-*/` | Generators (Tier 1: Rust-only; Tier 2: Blender subprocess) |
| `crates/speccade-lint/` | Semantic quality lint rules + analyzers |
| `crates/speccade-editor/` + `editor/` | Tauri editor + real-time preview |
| `crates/speccade-tests/` | Integration + determinism validation |
| `blender/speccade/` | Python package for Blender subprocess (handlers, skeleton, export) |
| `schemas/` | JSON schemas (derived; keep aligned with `speccade-spec`) |
| `specs/` | Starlark spec files (audio, texture, mesh, animation, sprite, vfx, ui, font) |
| `stdlib/` | Stdlib snapshot for drift detection |

## Coding Conventions

### Rust

- `#[serde(deny_unknown_fields)]` and `#[serde(rename_all = "snake_case")]` on all param types
- `BTreeMap` over `HashMap` in any output-affecting path (determinism)
- `anyhow` in CLI; typed errors (`BackendError` trait) in library crates
- `tempfile::tempdir()` for test isolation — never write to fixed paths
- All recipe param structs live under `crates/speccade-spec/src/recipe/<category>/`

### Python (Blender)

- Handler signature: `def handle_<mode>(params, report, out_root)`
- Scene lifecycle: clear → setup → work → metrics → report → export
- `orient_type='LOCAL'` for all `extrude_region_move` calls
- `apply_rotation_only(obj)` must come AFTER the extrusion loop and cap removal

### Starlark Stdlib

- Flat keyword args preferred over nested dicts
- Domain-prefixed names (`audio_envelope()`, `texture_noise_node()`)
- Use `#[starlark_module]` macro; register in domain's `register()` function

## Common Pitfalls

1. **Forgetting `is_backend_available()`** — dispatch works but `speccade doctor` and tests report the backend as unavailable
2. **Missing `try_parse_params()` arm** — params pass validation without actual type checking
3. **Stale `dispatch_generate_profiled()`** — profiled mode crashes on the new recipe kind
4. **`HashMap` in Tier 1 output paths** — causes nondeterministic output
5. **Blender extrusion with GLOBAL orient_type** — diagonal bones extrude along world Z instead of bone axis
6. **Schema drift** — `speccade-spec` types change without `schemas/` update
7. **Stdlib snapshot drift** — function signature changes without running `stdlib_snapshot` test

## Determinism Guardrails (Tier 1)

- Given the same validated spec + seed, output must be **byte-identical** across runs.
- Avoid: wall clock time, OS RNG, thread timing, filesystem ordering.
- Use: stable iteration order (`BTreeMap`/sorted keys), explicit rounding/quantization.
- Tier 2 (Blender): validated by metrics, not byte-identical. Output can vary by OS/Blender version.

## Quick Commands

```bash
# Tests
cargo test --workspace                          # All tests
cargo test -p speccade-tests                    # Integration + determinism
cargo test -p speccade-tests --test stdlib_snapshot  # Stdlib drift check

# Verify (CI mirror)
./scripts/verify.ps1                            # Windows
./scripts/verify.sh                             # macOS/Linux

# CLI
cargo run -p speccade-cli -- --help
cargo run -p speccade-cli -- eval --spec file.star --pretty
cargo run -p speccade-cli -- validate --spec file.star --budget strict
cargo run -p speccade-cli -- generate --spec file.star --out-root ./out --budget strict
cargo run -p speccade-cli -- preview-grid --spec file.star
cargo run -p speccade-cli -- stdlib dump --format json
```
