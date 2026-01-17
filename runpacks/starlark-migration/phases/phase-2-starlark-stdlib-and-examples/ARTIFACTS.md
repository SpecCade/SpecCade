# Phase 2 Artifacts

## Phase-produced artifacts

### Research outputs
| File | Description |
|------|-------------|
| `research.md` | Notes on existing asset types, stdlib patterns, LLM-friendly API design |
| `risks.md` | Identified risks and mitigations |
| `questions.md` | Blocking questions (if any) |

### Planning outputs
| File | Description |
|------|-------------|
| `plan.md` | Implementation plan with ordered tasks |
| `interfaces.md` | stdlib function signatures, error codes |
| `test_plan.md` | Golden test strategy, coverage requirements |

### Implementation outputs
| File | Description |
|------|-------------|
| `implementation_log.md` | Chronological log of changes made |
| `diff_summary.md` | Summary of files changed and why |

### Validation outputs
| File | Description |
|------|-------------|
| `validation.md` | Test run results and analysis |
| `failures.md` | Failure triage (if any failures) |

### Quality outputs
| File | Description |
|------|-------------|
| `quality.md` | Refactoring notes, documentation improvements |
| `followups.md` | Deferred items for Phase 3+ (optional) |

---

## Decision log

Record any deviations from `ARCHITECTURE_PROPOSAL.md` or scope changes here.

| Date | Decision | Rationale |
|------|----------|-----------|
| _TBD_ | _Example: stdlib function naming convention_ | _Chose snake_case for consistency with Rust_ |

---

## Code artifacts (expected)

After implementation, these repo paths should contain new/modified code:

### stdlib implementation
| Path | Change type | Description |
|------|-------------|-------------|
| `crates/speccade-cli/src/stdlib/` | New | stdlib module directory |
| `crates/speccade-cli/src/stdlib/mod.rs` | New | stdlib entry point |
| `crates/speccade-cli/src/stdlib/audio.rs` | New | Audio constructors (synth, tracker, etc.) |
| `crates/speccade-cli/src/stdlib/texture.rs` | New | Texture constructors (noise, gradient, etc.) |
| `crates/speccade-cli/src/stdlib/mesh.rs` | New | Mesh constructors (primitives, transforms) |

### Examples
| Path | Change type | Description |
|------|-------------|-------------|
| `packs/examples/audio_synth_basic.star` | New | Basic audio example |
| `packs/examples/texture_noise_perlin.star` | New | Texture example |
| `packs/examples/mesh_primitive_cube.star` | New | Mesh example |

### Golden tests
| Path | Change type | Description |
|------|-------------|-------------|
| `golden/starlark/` | New | Golden test directory for .star files |
| `golden/starlark/audio_synth_basic.star` | New | Test fixture |
| `golden/starlark/audio_synth_basic.expected.json` | New | Expected canonical IR |
| `crates/speccade-tests/tests/golden_starlark.rs` | New or Modified | Golden test runner |

### CLI diagnostics
| Path | Change type | Description |
|------|-------------|-------------|
| `crates/speccade-cli/src/diagnostics.rs` | New | Diagnostic codes and JSON output |
| `docs/error-codes.md` | New | Error code reference |

### Documentation
| Path | Change type | Description |
|------|-------------|-------------|
| `docs/starlark-authoring.md` | New | Starlark authoring guide |
| `docs/stdlib-reference.md` | New | stdlib function reference |

_Exact paths may vary based on planning phase decisions._
