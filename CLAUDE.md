# SpecCade - Claude Code Instructions

## Purpose

SpecCade is a deterministic asset pipeline. It validates a JSON `Spec` and generates assets via backends.

## Start Here

- `README.md` — usage and examples
- `ARCHITECTURE.md` — crate map + determinism model
- `PARITY_MATRIX.md` — what “deterministic” means per backend/tier

## Ground Rules (High Signal)

- Treat `crates/speccade-spec/` as the source of truth for the spec format (types + validation + hashing).
- Tier 1 backends (Rust-only) should be byte-identical for the same spec hash/seed.
- Tier 2 backends (external tools like Blender) should have explicit validation rules (don’t claim byte-identical output unless enforced).

## Common Workflows

- **Add/change spec fields:** update `speccade-spec` + `schemas/` + `speccade-tests` (and `golden/` if needed).
- **Add a new backend feature:** keep the spec change minimal, implement in the backend crate, then add determinism/integration coverage.
- **Debug a determinism regression:** start from the failing golden/test case in `crates/speccade-tests/` and trace inputs/seed hashing in `speccade-spec`.

## Quick Commands

- `cargo test`
- `cargo test -p speccade-tests`
- `cargo run -p speccade-cli -- --help`

