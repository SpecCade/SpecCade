# SpecCade

Deterministic asset pipeline: Specs (JSON/Starlark) → artifacts (WAV/PNG/XM/IT/GLB) + reports.

## Hard Rules

1. **Determinism is sacred (Tier 1)** — no OS RNG, wall clock, thread timing, filesystem ordering. Use `BTreeMap`/sorted keys.
2. **`speccade-spec` is the SSOT** for the public contract. Change types/validation there first.
3. **Starlark `.star`** is the preferred authoring format, not raw JSON.
4. **`#[serde(deny_unknown_fields)]`** on all recipe param types.
5. **Parallel match blocks must stay in sync** — `recipe/mod.rs` has 6 (`as_str`, `parse_kind`, `asset_type_prefix`, `is_tier1`, `as_<recipe>`, `try_parse_params`); `dispatch/mod.rs` has 3 (`dispatch_generate`, `dispatch_generate_profiled`, `is_backend_available`).
6. **Blender Python**: `orient_type='LOCAL'` for extrude ops on non-vertical bones.
7. **Don't treat RFCs/old plan docs as authoritative** unless `docs/ROADMAP.md` references them.
8. **Use "deprecated"** not "legacy" in backward-compat terminology.

## Verify Your Work

```bash
cargo fmt --all && cargo clippy --workspace --all-features && cargo test --workspace
./scripts/verify.ps1   # Windows (full CI mirror)
./scripts/verify.sh    # macOS/Linux
cargo run -p speccade-cli -- validate --spec <path>
cargo run -p speccade-cli -- generate --spec <path> --out-root ./out
```

## Navigation by Task

| I need to... | Start here |
|---|---|
| Add a recipe kind | `AGENTS.md` § "Adding a Recipe Kind" |
| Add a backend feature | `AGENTS.md` § "Adding a Backend Feature" |
| Add a stdlib function | `AGENTS.md` § "Adding a Stdlib Function" |
| Fix Blender handler bug | `blender/speccade/handlers_*.py` + `AGENTS.md` § "Changing Blender Python" |
| Understand architecture | `ARCHITECTURE.md` |
| Understand spec contract | `crates/speccade-spec/`, `schemas/` |
| Author a spec | `docs/starlark-authoring.md`, `docs/stdlib-reference.md` |
| See planned work | `docs/ROADMAP.md` |
| Check backend coverage | `PARITY_MATRIX.md` |

## Single Source of Truth

| What | Where |
|---|---|
| Spec contract + validation | `crates/speccade-spec/` and `speccade validate` |
| CLI surface | `speccade --help` |
| Stdlib surface | `speccade stdlib dump --format json` |
| Planned work | `docs/ROADMAP.md` |
| JSON schemas (editor assistance) | `schemas/` — keep aligned with `speccade-spec` |

## Local Development

If `speccade` is not installed, substitute: `cargo run -p speccade-cli -- <args>`

## Deep Context

- **`AGENTS.md`** — change recipes, coding conventions, common pitfalls
- **`ARCHITECTURE.md`** — crate map, module dependencies, compilation pipeline, determinism model
