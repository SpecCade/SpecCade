# Phase 4 Implementation Prompt

## Objective

Add stdlib functions for all missing IR types.

## Implementation Strategy

### Sub-phase 4d: Budget Enforcement (DO FIRST)

1. In `crates/speccade-spec/src/validation/mod.rs`:
   - Change `_budget: &BudgetProfile` to `budget: &BudgetProfile`
   - Pass budget to recipe validation functions

2. In `crates/speccade-cli/src/commands/validate.rs`:
   - Add `--budget` CLI flag with choices: default, strict, zx-8bit
   - Pass selected profile to validation

3. In `crates/speccade-cli/src/commands/generate.rs`:
   - Add same `--budget` flag
   - Pass to validation before generation

4. Update recipe validators to use profile values instead of hardcoded constants.

### Sub-phase 4a: Audio Parity

For each missing synthesis type, add a function following this pattern:

```rust
fn synth_type_name(
    // Required params from IR type
    param1: Value,
    param2: Value,
    // Optional params with defaults
    param3: Option<Value>,
) -> Result<Value, Error> {
    // Validate params
    // Build dict matching IR serialization
    Ok(heap.alloc(dict))
}
```

**Synthesis types to add** (24):
- `am_synth()` - AmSynth
- `ring_mod_synth()` - RingModSynth
- `additive()` - Additive
- `multi_oscillator()` - MultiOscillator
- `granular()` - Granular
- `wavetable()` - Wavetable
- `pd_synth()` - PdSynth
- `modal()` - Modal
- `vocoder()` - Vocoder
- `formant_synth()` - Formant
- `vector()` - Vector
- `supersaw_unison()` - SupersawUnison
- `waveguide()` - Waveguide
- `bowed_string()` - BowedString
- `membrane_drum()` - MembraneDrum
- `feedback_fm()` - FeedbackFm
- `comb_filter_synth()` - CombFilterSynth
- `pulsar()` - Pulsar
- `vosim()` - Vosim
- `spectral_freeze()` - SpectralFreeze
- `pitched_body()` - PitchedBody
- `metallic()` - Metallic

**Filter types to add** (9):
- `bandpass()` - Bandpass
- `notch()` - Notch
- `allpass()` - Allpass
- `comb_filter()` - Comb
- `formant_filter()` - Formant
- `ladder()` - Ladder
- `shelf_low()` - ShelfLow
- `shelf_high()` - ShelfHigh

**Effect types to add** (16):
- `parametric_eq()` - ParametricEq
- `chorus()` - Chorus
- `phaser()` - Phaser
- `bitcrush()` - Bitcrush
- `waveshaper()` - Waveshaper
- `flanger()` - Flanger
- `limiter()` - Limiter
- `gate_expander()` - GateExpander
- `stereo_widener()` - StereoWidener
- `multi_tap_delay()` - MultiTapDelay
- `tape_saturation()` - TapeSaturation
- `transient_shaper()` - TransientShaper
- `auto_filter()` - AutoFilter
- `cabinet_sim()` - CabinetSim
- `rotary_speaker()` - RotarySpeaker
- `ring_modulator()` - RingModulator
- `granular_delay()` - GranularDelay

**LFO/Modulation:**
- `lfo()` - Create LfoConfig
- `pitch_envelope()` - Create PitchEnvelope
- Update `audio_layer()` to accept LFO params

**Fix partial implementations:**
- `oscillator()` - Add `detune`, `duty` params
- `reverb()` - Add `width` param
- `delay()` - Add `ping_pong` param
- `compressor()` - Add `makeup_db` param

### Sub-phase 4b: Texture & Mesh Parity

**Texture operations to add** (10):
- `add_node()` - Add blend
- `multiply_node()` - Multiply blend
- `lerp_node()` - Linear interpolation
- `clamp_node()` - Clamp values
- `stripes_node()` - Stripe pattern
- `checkerboard_node()` - Checkerboard pattern
- `grayscale_node()` - ToGrayscale
- `palette_node()` - Palette quantization
- `compose_rgba_node()` - ComposeRgba
- `normal_from_height_node()` - NormalFromHeight

**Fix gradient_node():**
- Add `center`, `inner`, `outer` params for radial gradients

**Mesh modifiers to add** (4):
- `edge_split_modifier()` - EdgeSplit
- `mirror_modifier()` - Mirror
- `array_modifier()` - Array
- `solidify_modifier()` - Solidify

**Fix bevel_modifier():**
- Add `angle_limit` param

**Mesh features to add:**
- `material_slot()` - MaterialSlot definition
- `uv_projection()` - UvProjection settings
- `export_settings()` - MeshExportSettings

### Sub-phase 4c: Music Stdlib

Create new file: `crates/speccade-cli/src/compiler/stdlib/music.rs`

Add functions:
- `tracker_instrument()` - Define instrument
- `tracker_pattern()` - Define pattern
- `tracker_song()` - Create TrackerSong spec
- `compose_pattern()` - Compose DSL pattern
- `compose_song()` - Compose DSL song

Register in `stdlib/mod.rs`.

## Testing

For each new function:
1. Create `golden/starlark/{domain}_{type}.star`
2. Ensure it produces valid IR
3. Run `cargo test -p speccade-tests`

## Deliverables

- `implementation_log.md` - What was added
- `diff_summary.md` - Files changed
