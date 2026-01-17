# Phase 3 Artifacts: Budgets, Caching, and Hardening

## Phase-Produced Artifacts

| Artifact | Path | Status |
|----------|------|--------|
| Research notes | `research.md` | Pending |
| Risk analysis | `risks.md` | Pending |
| Blocking questions | `questions.md` | Pending (if needed) |
| Implementation plan | `plan.md` | Pending |
| Interface definitions | `interfaces.md` | Pending |
| Test plan | `test_plan.md` | Pending |
| Implementation log | `implementation_log.md` | Pending |
| Diff summary | `diff_summary.md` | Pending |
| Validation results | `validation.md` | Pending |
| Failure triage | `failures.md` | Pending (if needed) |
| Quality review | `quality.md` | Pending |
| Follow-up items | `followups.md` | Pending (if needed) |

## Expected Code Artifacts

| Component | Location | Purpose |
|-----------|----------|---------|
| Budget types | `crates/speccade-spec/src/budget.rs` | Budget definitions + profile loading |
| Validation budgets | `crates/speccade-spec/src/validation/budgets.rs` | Budget enforcement at validation |
| Provenance types | `crates/speccade-spec/src/provenance.rs` | Source hash + stdlib version |
| Report extension | `crates/speccade-cli/src/report.rs` | Provenance in reports |
| Cache key module | `crates/speccade-cli/src/cache.rs` | Caching key generation |
| Idempotence tests | `crates/speccade-tests/src/idempotence.rs` | Canonicalization tests |
| Fuzz-ish tests | `crates/speccade-tests/src/fuzz.rs` | Property-based validation |

## Decision Log

| Date | Decision | Rationale |
|------|----------|-----------|
| _TBD_ | _Decisions will be logged here_ | _Rationale_ |

## Architecture References

- `ARCHITECTURE_PROPOSAL.md` Section C: Validation (schema + invariants + budgets)
- `ARCHITECTURE_PROPOSAL.md` Section E: Output + determinism report
- `PARITY_MATRIX.md`: Tier 1/2 determinism expectations
