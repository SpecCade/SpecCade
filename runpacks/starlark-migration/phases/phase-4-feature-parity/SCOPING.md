# Phase 4 Scoping: Feature Parity Completion

## Objective

Achieve full feature parity between JSON IR and Starlark stdlib. After Phase 4, any asset expressible in JSON IR should be expressible via Starlark stdlib functions.

## Allowed file globs

```
crates/speccade-cli/**
crates/speccade-spec/**
crates/speccade-tests/**
docs/**
golden/**
schemas/**
Cargo.toml
Cargo.lock
```

## Must-not-touch guidance

| Area | Reason |
|------|--------|
| `crates/speccade-backend-*/**` | Backends remain IR-only consumers |
| `speccade_spec::Spec` struct shape | IR contract is frozen |
| Existing stdlib function signatures | Backwards compatible |

## Acceptance Criteria

### 4a: Audio Parity
- [ ] All 29 `Synthesis` variants have stdlib functions
- [ ] All 11 `Filter` variants have stdlib functions
- [ ] All 19 `Effect` variants have stdlib functions
- [ ] LFO/modulation has stdlib support (`lfo()`, targets)
- [ ] Existing functions updated with missing params (detune, duty, ping_pong, etc.)

### 4b: Texture & Mesh Parity
- [ ] All 17 `TextureProceduralOp` variants have stdlib functions
- [ ] All 7 `MeshModifier` variants have stdlib functions
- [ ] Mesh features: MaterialSlot, UvProjection helpers

### 4c: Music Stdlib
- [ ] New `music.rs` stdlib module
- [ ] TrackerInstrument definition helpers
- [ ] Pattern/sequence helpers
- [ ] Format selection (xm/it)

### 4d: Budget Enforcement
- [ ] BudgetProfile wired to validation (not unused `_budget`)
- [ ] CLI `--budget` flag for profile selection
- [ ] Hardcoded constants replaced with profile values

### Tests
- [ ] Golden tests for all new stdlib functions
- [ ] Integration tests for budget profile selection
- [ ] At least one example per synthesis type

## Priority Order

1. **4d: Budget Enforcement** - Quick win, architectural fix
2. **4a: Audio Parity** - Largest gap, highest user value
3. **4b: Texture & Mesh Parity** - Medium effort
4. **4c: Music Stdlib** - New module, can be done in parallel

## Safety Notes

### Determinism
- All new stdlib functions MUST be deterministic
- No random, time, network, or IO

### Backwards Compatibility
- Existing stdlib functions must retain signatures
- New optional params only (with defaults)

### Testing
- Each new function requires at least one golden test
- Test file naming: `{domain}_{type}.star`
