# Phase 1 Status: Compiler Pipeline + .star Input

**Status: COMPLETE** (2026-01-17)

## Stage completion

- [x] **Research** - Understand existing CLI, spec crate, and Starlark integration points
- [x] **Plan** - Design compiler pipeline, new `eval` command, input dispatch
- [x] **Implement** - Add Starlark parsing, compiler module, CLI changes
- [x] **Validate** - Run test suites, verify JSON compatibility
- [x] **Quality** - Refactor, improve error messages, documentation

---

## Acceptance criteria

From `PHASES.yaml`:

- [x] CLI accepts `.json` (existing) AND `.star` inputs for validate/generate
- [x] New command: `speccade eval <spec.star|ir.json>` prints canonical IR JSON
- [x] Backends consume only canonical `speccade_spec::Spec` (no Starlark in backends)
- [x] Determinism hashes are computed on canonical IR (post expansion)
- [x] No breaking changes for JSON users

---

## Current blockers

None. Phase 1 is complete.

---

## Validation results

All test suites passed:
- speccade-spec: 547 unit tests, 12 doc tests
- speccade-cli: 105 unit tests, 16 integration tests
- speccade-tests: 87 unit tests, 104 integration tests

---

## Notes

- Starlark stdlib kept minimal (standard dialect only)
- Compose expansion deferred to Phase 1b/2 (current architecture works)
- Safety limits: 30s timeout, recursion disabled, no load() statements
- Feature gate: Starlark support optional via `starlark` feature (enabled by default)
