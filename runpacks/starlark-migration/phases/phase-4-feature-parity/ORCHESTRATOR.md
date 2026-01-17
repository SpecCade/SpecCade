# Phase 4 Orchestrator Guide

## Phase Objective

Achieve full Starlark stdlib coverage of JSON IR types. Current coverage: ~15-20%. Target: 100%.

## Stage Flow

```
00_research -> 10_plan -> 20_implement -> 30_validate -> 40_quality
```

## Sub-phase Strategy

Phase 4 is divided into parallel workstreams:

| Sub-phase | Focus | Estimated Effort |
|-----------|-------|------------------|
| 4d | Budget enforcement | Small (1-2 files) |
| 4a | Audio parity | Large (50+ functions) |
| 4b | Texture/mesh parity | Medium (15+ functions) |
| 4c | Music stdlib | Medium (new module) |

**Recommendation**: Start 4d immediately (quick architectural fix), then parallelize 4a/4b/4c.

## Key Files

### Stdlib to extend
- `crates/speccade-cli/src/compiler/stdlib/audio.rs`
- `crates/speccade-cli/src/compiler/stdlib/texture.rs`
- `crates/speccade-cli/src/compiler/stdlib/mesh.rs`
- `crates/speccade-cli/src/compiler/stdlib/music.rs` (NEW)

### Budget enforcement
- `crates/speccade-spec/src/validation/budgets.rs`
- `crates/speccade-spec/src/validation/mod.rs`
- `crates/speccade-cli/src/commands/validate.rs`
- `crates/speccade-cli/src/commands/generate.rs`

### IR types (source of truth)
- `crates/speccade-spec/src/audio/synthesis.rs`
- `crates/speccade-spec/src/audio/effect.rs`
- `crates/speccade-spec/src/audio/filter.rs`
- `crates/speccade-spec/src/texture/procedural.rs`
- `crates/speccade-spec/src/mesh/modifier.rs`
- `crates/speccade-spec/src/music/tracker.rs`

## Implementation Pattern

For each missing stdlib function:

1. **Read IR type** from `speccade-spec` to understand all fields
2. **Create Starlark function** with matching parameters
3. **Return JSON dict** that matches IR serialization
4. **Add validation** for parameter constraints
5. **Write golden test** in `golden/starlark/`
6. **Run tests** to verify

## Completion Checklist

- [ ] All `Synthesis` variants (29 total)
- [ ] All `Filter` variants (11 total)
- [ ] All `Effect` variants (19 total)
- [ ] LFO/modulation support
- [ ] All `TextureProceduralOp` variants (17 total)
- [ ] All `MeshModifier` variants (7 total)
- [ ] Mesh features (materials, UV, export)
- [ ] Music stdlib module
- [ ] Budget enforcement wired
- [ ] Golden tests complete
- [ ] Documentation updated

## Exit Criteria

Phase 4 is complete when:
1. `cargo test` passes
2. Every IR type has a corresponding stdlib function
3. Golden tests exist for each stdlib function
4. Budget profiles are actually enforced (not `_budget`)
