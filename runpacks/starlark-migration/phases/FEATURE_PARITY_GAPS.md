# Starlark Feature Parity Gaps Analysis

**Generated**: 2026-01-18
**Status**: Verified via deep code inspection (not documentation review)

## Executive Summary

The Starlark stdlib covers approximately **15-20%** of JSON IR capabilities. The implementation status documents claimed "COMPLETE" but this refers to the scoped acceptance criteria, not full feature parity with the JSON IR.

---

## Gap Analysis by Domain

### 1. AUDIO SYNTHESIS (83% Missing)

**Implemented (5 of 29):**
- `Oscillator` (via `oscillator()`) - PARTIAL: missing `detune`, `duty` params
- `FmSynth` (via `fm_synth()`)
- `KarplusStrong` (via `karplus_strong()`)
- `NoiseBurst` (via `noise_burst()`)

**Missing (24 synthesis types):**
| Type | Priority | Complexity |
|------|----------|------------|
| `AmSynth` | HIGH | Low |
| `RingModSynth` | HIGH | Low |
| `Additive` | HIGH | Medium |
| `MultiOscillator` | HIGH | Medium |
| `Granular` | HIGH | High |
| `Wavetable` | HIGH | High |
| `PdSynth` | MEDIUM | Medium |
| `Modal` | MEDIUM | High |
| `Vocoder` | MEDIUM | High |
| `Formant` | MEDIUM | High |
| `Vector` | LOW | High |
| `SupersawUnison` | HIGH | Medium |
| `Waveguide` | LOW | High |
| `BowedString` | LOW | High |
| `MembraneDrum` | MEDIUM | Medium |
| `FeedbackFm` | MEDIUM | Medium |
| `CombFilterSynth` | MEDIUM | Low |
| `Pulsar` | LOW | Medium |
| `Vosim` | LOW | Medium |
| `SpectralFreeze` | LOW | Medium |
| `PitchedBody` | LOW | Low |
| `Metallic` | MEDIUM | Medium |

### 2. AUDIO FILTERS (82% Missing)

**Implemented (2 of 11):**
- `Lowpass` (via `lowpass()`)
- `Highpass` (via `highpass()`)

**Missing (9 filter types):**
| Type | Priority | Complexity |
|------|----------|------------|
| `Bandpass` | HIGH | Low |
| `Notch` | HIGH | Low |
| `Allpass` | MEDIUM | Low |
| `Comb` | MEDIUM | Low |
| `Formant` | MEDIUM | Medium |
| `Ladder` | HIGH | Low |
| `ShelfLow` | MEDIUM | Low |
| `ShelfHigh` | MEDIUM | Low |

### 3. AUDIO EFFECTS (84% Missing)

**Implemented (3 of 19):**
- `Reverb` (via `reverb()`) - PARTIAL: missing `width`
- `Delay` (via `delay()`) - PARTIAL: missing `ping_pong`
- `Compressor` (via `compressor()`) - PARTIAL: missing `makeup_db`

**Missing (16 effect types):**
| Type | Priority | Complexity |
|------|----------|------------|
| `Chorus` | HIGH | Low |
| `Phaser` | HIGH | Medium |
| `Bitcrush` | HIGH | Low |
| `Waveshaper` | MEDIUM | Low |
| `Flanger` | MEDIUM | Medium |
| `Limiter` | HIGH | Low |
| `GateExpander` | MEDIUM | Medium |
| `StereoWidener` | LOW | Low |
| `MultiTapDelay` | MEDIUM | Medium |
| `TapeSaturation` | LOW | Medium |
| `TransientShaper` | MEDIUM | Medium |
| `AutoFilter` | MEDIUM | Medium |
| `CabinetSim` | LOW | Low |
| `RotarySpeaker` | LOW | Medium |
| `RingModulator` | MEDIUM | Low |
| `GranularDelay` | LOW | High |
| `ParametricEq` | HIGH | Medium |

### 4. AUDIO MODULATION (100% Missing)

**No LFO/modulation stdlib functions exist.**

Required:
- `lfo()` function to create `LfoConfig`
- `pitch_envelope()` function
- Support for all `ModulationTarget` variants:
  - Pitch, Volume, FilterCutoff, Pan, PulseWidth
  - FmIndex, GrainSize, GrainDensity
  - DelayTime, ReverbSize, DistortionDrive

### 5. TEXTURE OPERATIONS (59% Missing)

**Implemented (7 of 17):**
- `Constant` (via `constant_node()`)
- `Noise` (via `noise_node()`)
- `Gradient` (via `gradient_node()`) - PARTIAL: missing radial params
- `Invert` (via `invert_node()`)
- `Threshold` (via `threshold_node()`)
- `ColorRamp` (via `color_ramp_node()`)

**Missing (10 operations):**
| Type | Priority | Complexity |
|------|----------|------------|
| `Add` | HIGH | Low |
| `Multiply` | HIGH | Low |
| `Lerp` | HIGH | Low |
| `Clamp` | MEDIUM | Low |
| `Stripes` | MEDIUM | Low |
| `Checkerboard` | MEDIUM | Low |
| `ToGrayscale` | LOW | Low |
| `Palette` | LOW | Medium |
| `ComposeRgba` | MEDIUM | Medium |
| `NormalFromHeight` | MEDIUM | Medium |

### 6. MESH MODIFIERS (57% Missing)

**Implemented (3 of 7):**
- `Bevel` (via `bevel_modifier()`) - PARTIAL: missing `angle_limit`
- `Subdivision` (via `subdivision_modifier()`)
- `Decimate` (via `decimate_modifier()`)

**Missing (4 modifiers):**
| Type | Priority | Complexity |
|------|----------|------------|
| `EdgeSplit` | MEDIUM | Low |
| `Mirror` | HIGH | Low |
| `Array` | HIGH | Low |
| `Solidify` | MEDIUM | Low |

### 7. MESH FEATURES (100% Missing)

Required:
- `MaterialSlot` definition helpers
- `MeshExportSettings` configuration
- `MeshConstraints` for validation
- `UvProjection` methods (box, cylinder, sphere, smart, lightmap)

### 8. MUSIC DOMAIN (100% Missing)

**No stdlib support exists for:**
- `music.tracker_song_v1` recipe
- `music.tracker_song_compose_v1` recipe
- `TrackerInstrument` definitions
- Pattern/sequence helpers
- `TrackerFormat` (xm, it)
- `TrackerLoopMode`
- Compose DSL

---

## Budget Enforcement Gap

The `BudgetProfile` system is defined but **not enforced**:

**Location**: `crates/speccade-spec/src/validation/budgets.rs`

**Issue**: In `validate_for_generate_with_budget()`, the budget parameter is `_budget` (unused).

**Comment in code**: "TODO: Migrate inline constants in those modules to use BudgetProfile"

**Impact**: Budget profiles (`default`, `strict`, `zx-8bit`) exist but backend validation uses hardcoded constants.

---

## Test Coverage Gaps

### Golden Tests Missing
- `karplus_strong()` synthesis
- `noise_burst()` synthesis
- `highpass()` filter
- `compressor()` effect
- `decimate_modifier()` mesh modifier
- All texture operations beyond noise/gradient/threshold/color_ramp
- Music asset type
- Animation asset type
- Character asset type

### Integration Tests Missing
- Budget profile selection via CLI
- LFO modulation (none exist in IR)
- Multi-layer audio with effects chains
- Complex texture graphs with blend operations

---

## Recommended Phase 4 Scope

### Phase 4a: Audio Parity (HIGH priority)
- Add 24 missing synthesis types
- Add 9 missing filter types
- Add 16 missing effect types
- Add LFO/modulation support
- Fix partial implementations (missing params)

### Phase 4b: Texture & Mesh Parity (MEDIUM priority)
- Add 10 missing texture operations
- Add 4 missing mesh modifiers
- Add mesh features (materials, UV, export settings)

### Phase 4c: Music Stdlib (MEDIUM priority)
- Create music stdlib module
- Add tracker instrument helpers
- Add pattern/sequence DSL
- Support compose workflow

### Phase 4d: Budget Enforcement (HIGH priority)
- Wire BudgetProfile to validation
- Add CLI `--budget` flag
- Replace hardcoded constants with profile values

---

## Files to Modify

### Stdlib Extensions
- `crates/speccade-cli/src/compiler/stdlib/audio.rs` - Add synthesis/filter/effect functions
- `crates/speccade-cli/src/compiler/stdlib/texture.rs` - Add blend/pattern operations
- `crates/speccade-cli/src/compiler/stdlib/mesh.rs` - Add modifiers and features
- `crates/speccade-cli/src/compiler/stdlib/music.rs` - NEW FILE for music stdlib

### Budget Enforcement
- `crates/speccade-spec/src/validation/mod.rs` - Wire BudgetProfile
- `crates/speccade-cli/src/commands/validate.rs` - Add --budget flag
- `crates/speccade-cli/src/commands/generate.rs` - Add --budget flag

### Tests
- `crates/speccade-tests/tests/starlark_input.rs` - Add coverage for new functions
- `golden/starlark/` - Add golden tests for new asset types
