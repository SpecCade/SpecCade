# Phase 4 Continuation Prompt

## Context

Phase 1-3 are complete but only achieved ~15-20% feature parity between JSON IR and Starlark stdlib. Phase 4 will complete the remaining ~80%.

## Gap Summary

See `FEATURE_PARITY_GAPS.md` for full details. Key gaps:
- 24 missing audio synthesis types
- 9 missing audio filter types
- 16 missing audio effect types
- 13 missing LFO/modulation support
- 10 missing texture operations
- 4 missing mesh modifiers
- Entire music domain missing
- Budget profiles defined but not enforced

## Recommended Execution Order

### 1. Budget Enforcement (4d) - START HERE
Quick architectural fix. Estimated: 2-3 files, ~100 lines.

```
Read: crates/speccade-spec/src/validation/budgets.rs (understand BudgetProfile)
Read: crates/speccade-spec/src/validation/mod.rs (find unused _budget param)
Edit: Wire budget to validation, add CLI flag
Test: cargo test -p speccade-spec -p speccade-cli
```

### 2. Audio Parity (4a) - LARGEST EFFORT
50+ new stdlib functions. Can be done incrementally.

Priority order:
1. Fix partial implementations (oscillator, reverb, delay, compressor)
2. Add LFO support (enables modulation for all synths)
3. Add common synthesis (additive, wavetable, granular)
4. Add filters (bandpass, notch, ladder)
5. Add effects (chorus, phaser, bitcrush, limiter)
6. Add remaining synthesis types

### 3. Texture & Mesh Parity (4b) - MEDIUM EFFORT
~18 new functions.

### 4. Music Stdlib (4c) - NEW MODULE
Can be done in parallel with audio work.

## To Begin Implementation

1. Run the research prompt: `prompts/00_research.md`
2. Document findings in `research.md`
3. Create detailed plan in `plan.md`
4. Execute implementation via `prompts/20_implement.md`
5. Validate with `cargo test`
6. Update `STATUS.md` as you progress

## Success Criteria

- `cargo test` passes
- `cargo clippy --all-targets` clean
- Every IR type has stdlib function
- Golden tests for all new functions
- Budget profiles actually enforced
