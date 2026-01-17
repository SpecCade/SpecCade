# Phase 2 Implementation Plan: Starlark Stdlib and Examples

## Overview

This plan implements a minimal, LLM-friendly Starlark stdlib for SpecCade. The stdlib provides helper functions that emit canonical IR-compatible structures, reducing boilerplate and improving authoring ergonomics.

## Design Principles

1. **Flat, explicit parameters** - No magic defaults; LLMs understand keyword args well
2. **Composable** - Functions return dicts that can be modified or combined
3. **Deterministic** - No random, time, or IO functions
4. **Minimal** - Start with 15-20 core functions covering 80% of use cases
5. **Domain-prefixed** - `audio_*`, `texture_*`, `mesh_*` to avoid conflicts

---

## Step 1: Stdlib Module Structure

**Complexity:** Low
**Dependencies:** Phase 1 compiler (complete)

### Files to Create

```
crates/speccade-cli/src/compiler/
  stdlib/
    mod.rs          # Module registration, re-exports
    audio.rs        # Audio synthesis helpers
    texture.rs      # Texture graph helpers
    mesh.rs         # Mesh primitive helpers
    core.rs         # Spec/output scaffolding
    validation.rs   # Shared validation utilities
```

### Files to Modify

- `crates/speccade-cli/src/compiler/mod.rs` - Add `pub mod stdlib;`
- `crates/speccade-cli/src/compiler/eval.rs` - Wire stdlib into `GlobalsBuilder`

### Implementation Notes

1. Create `stdlib/mod.rs` with a single `#[starlark_module]` function `speccade_stdlib`
2. Import and call domain-specific registration functions:
   ```rust
   #[starlark_module]
   fn speccade_stdlib(builder: &mut GlobalsBuilder) {
       audio::register(builder);
       texture::register(builder);
       mesh::register(builder);
       core::register(builder);
   }
   ```
3. Modify `eval.rs` line 51:
   ```rust
   // FROM: let globals = Globals::standard();
   // TO:
   let globals = GlobalsBuilder::standard()
       .with(stdlib::speccade_stdlib)
       .build();
   ```

---

## Step 2: Audio Stdlib Functions

**Complexity:** Medium
**Dependencies:** Step 1

### Functions to Implement

| Function | Returns | Purpose |
|----------|---------|---------|
| `envelope(attack, decay, sustain, release)` | dict | ADSR envelope |
| `oscillator(frequency, waveform, sweep_to, curve)` | dict | Oscillator synthesis |
| `fm_synth(carrier, modulator, index, sweep_to)` | dict | FM synthesis |
| `noise_burst(noise_type, filter)` | dict | Noise synthesis |
| `karplus_strong(frequency, decay, blend)` | dict | Plucked string |
| `lowpass(cutoff, resonance, sweep_to)` | dict | Lowpass filter |
| `highpass(cutoff, resonance, sweep_to)` | dict | Highpass filter |
| `audio_layer(synthesis, envelope, volume, pan, filter, lfo)` | dict | Complete layer |
| `reverb(decay, wet, room_size)` | dict | Reverb effect |
| `delay(time_ms, feedback, wet)` | dict | Delay effect |
| `compressor(threshold_db, ratio, attack_ms, release_ms)` | dict | Compressor |

### Files to Create

- `crates/speccade-cli/src/compiler/stdlib/audio.rs`

### Validation Requirements

- `frequency` must be positive
- `volume`, `pan` must be in valid ranges
- `waveform` must be valid enum string
- Return S101/S102/S103 errors with helpful messages

### IR Mapping

Each function returns a dict that matches the corresponding `speccade_spec` struct when serialized:

```python
# oscillator() output matches Synthesis::Oscillator
{
    "type": "oscillator",
    "waveform": "sine",
    "frequency": 440.0,
    "freq_sweep": None  # omitted if None
}
```

---

## Step 3: Texture Stdlib Functions

**Complexity:** Medium
**Dependencies:** Step 1

### Functions to Implement

| Function | Returns | Purpose |
|----------|---------|---------|
| `noise_node(id, algorithm, scale, octaves)` | dict | Noise texture node |
| `gradient_node(id, direction, start, end)` | dict | Gradient node |
| `constant_node(id, value)` | dict | Constant value node |
| `threshold_node(id, input, threshold)` | dict | Threshold operation |
| `invert_node(id, input)` | dict | Invert operation |
| `color_ramp_node(id, input, ramp)` | dict | Color mapping |
| `texture_graph(resolution, tileable, nodes)` | dict | Complete graph |

### Files to Create

- `crates/speccade-cli/src/compiler/stdlib/texture.rs`

### Validation Requirements

- `id` must be non-empty string
- `resolution` must be positive integers
- `algorithm` must be valid enum string
- Node references must be strings (validation of graph happens at spec level)

---

## Step 4: Mesh Stdlib Functions

**Complexity:** Low
**Dependencies:** Step 1

### Functions to Implement

| Function | Returns | Purpose |
|----------|---------|---------|
| `mesh_primitive(primitive, dimensions)` | dict | Base primitive |
| `bevel_modifier(width, segments)` | dict | Bevel modifier |
| `subdivision_modifier(levels)` | dict | Subdivision modifier |
| `decimate_modifier(ratio)` | dict | Decimate modifier |
| `mesh_recipe(primitive, dimensions, modifiers)` | dict | Complete recipe |

### Files to Create

- `crates/speccade-cli/src/compiler/stdlib/mesh.rs`

### Notes

- Mesh is Tier 2 (Blender-dependent), lower priority
- Keep minimal - just primitive shapes and common modifiers

---

## Step 5: Core Stdlib Functions

**Complexity:** Low
**Dependencies:** Step 1

### Functions to Implement

| Function | Returns | Purpose |
|----------|---------|---------|
| `spec(asset_id, asset_type, seed, outputs, recipe, description, tags)` | dict | Complete spec |
| `output(path, format, kind)` | dict | Output specification |

### Files to Create

- `crates/speccade-cli/src/compiler/stdlib/core.rs`

### Notes

- These are convenience wrappers; users can still write raw dicts
- Validate `asset_id` format, `seed` range

---

## Step 6: Example .star Files

**Complexity:** Low
**Dependencies:** Steps 2-5

### Examples to Create

```
golden/starlark/
  # Audio examples
  audio_synth_oscillator.star       # Simple sine wave with envelope
  audio_synth_fm.star               # FM synthesis example
  audio_synth_layered.star          # Multi-layer sound

  # Texture examples
  texture_noise_perlin.star         # Basic Perlin noise
  texture_graph_threshold.star      # Noise with threshold mask
  texture_color_ramp.star           # Noise with color mapping

  # Mesh examples
  mesh_primitive_cube.star          # Simple cube with bevel
  mesh_primitive_sphere.star        # Sphere with subdivision
```

### Example Content Pattern

```python
# audio_synth_oscillator.star
# Simple oscillator with envelope - demonstrates audio stdlib

spec(
    asset_id = "stdlib-audio-osc-01",
    asset_type = "audio",
    seed = 42,
    outputs = [output("sounds/osc.wav", "wav")],
    recipe = {
        "kind": "audio_v1",
        "params": {
            "duration_seconds": 0.5,
            "sample_rate": 44100,
            "layers": [
                audio_layer(
                    synthesis = oscillator(440, "sine"),
                    envelope = envelope(0.01, 0.1, 0.5, 0.2),
                    volume = 0.8,
                    pan = 0.0
                )
            ]
        }
    }
)
```

### Corresponding Expected Files

Each `.star` file gets a `.expected.json` with the canonical IR output.

---

## Step 7: Golden Tests

**Complexity:** Medium
**Dependencies:** Step 6

### Test Files to Create

```
crates/speccade-tests/tests/
  starlark_stdlib.rs       # Unit tests for stdlib functions
  starlark_golden.rs       # Golden IR equality tests
```

### Test Categories

1. **Stdlib Unit Tests** - Test individual function outputs
   ```rust
   #[test]
   fn test_envelope_defaults() {
       let result = eval_expr("envelope()");
       assert_eq!(result["attack"], 0.01);
   }
   ```

2. **Golden IR Tests** - Compare compiled .star to .expected.json
   ```rust
   #[test]
   fn golden_audio_synth_oscillator() {
       let result = compile_star("golden/starlark/audio_synth_oscillator.star");
       let expected = load_json("golden/starlark/audio_synth_oscillator.expected.json");
       assert_canonical_eq(result, expected);
   }
   ```

3. **Error Message Tests** - Verify error codes and messages
   ```rust
   #[test]
   fn test_oscillator_negative_frequency_error() {
       let result = eval_expr("oscillator(-440)");
       assert_error(result, "S103", "frequency must be positive");
   }
   ```

### Test Infrastructure

- Add `eval_expr()` helper to evaluate single expressions
- Add `assert_canonical_eq()` for JCS-canonical JSON comparison
- Reuse existing `DeterminismFixture` for multi-run tests

---

## Step 8: CLI Diagnostics Enhancement

**Complexity:** Low
**Dependencies:** Step 2-5

### Files to Modify

- `crates/speccade-cli/src/compiler/error.rs` - Add S-series error codes

### Error Code Range

| Range | Category | Description |
|-------|----------|-------------|
| S001-S009 | Compiler | Syntax, runtime, timeout errors |
| S101-S199 | Stdlib | Function argument validation |
| S201-S299 | Reserved | Future stdlib categories |

### New Error Variants

```rust
pub enum CompileError {
    // Existing...

    /// S101: Invalid stdlib function argument
    StdlibArgument {
        function: String,
        param: String,
        message: String,
    },

    /// S102: Type mismatch in stdlib function
    StdlibType {
        function: String,
        param: String,
        expected: String,
        got: String,
    },

    /// S103: Value out of range in stdlib function
    StdlibRange {
        function: String,
        param: String,
        range: String,
        got: String,
    },
}
```

### CLI Output Enhancement

Add `--json` flag support for machine-readable diagnostics:

```json
{
  "error": {
    "code": "S103",
    "function": "oscillator",
    "param": "frequency",
    "message": "must be positive",
    "got": "-440"
  }
}
```

---

## Step 9: Documentation

**Complexity:** Low
**Dependencies:** All previous steps

### Files to Create

```
docs/starlark/
  stdlib-reference.md     # Complete stdlib API reference
  examples.md             # Example walkthrough
  error-codes.md          # S-series error code reference
```

### Documentation Content

1. **stdlib-reference.md**
   - Function signatures with types
   - Parameter descriptions and defaults
   - Return value structure
   - Example usage for each function

2. **examples.md**
   - Annotated walkthrough of each example file
   - Common patterns and idioms
   - Migration guide from JSON to Starlark

3. **error-codes.md**
   - Complete S-series error code table
   - Example error messages
   - Resolution guidance for each code

---

## Implementation Order

```
Week 1:
  Step 1: Stdlib module structure
  Step 2: Audio stdlib (core functions)
  Step 5: Core stdlib (spec/output)

Week 2:
  Step 2: Audio stdlib (remaining functions)
  Step 3: Texture stdlib
  Step 4: Mesh stdlib
  Step 8: CLI diagnostics

Week 3:
  Step 6: Example files
  Step 7: Golden tests
  Step 9: Documentation
```

---

## Success Criteria

1. All stdlib functions pass unit tests
2. All example .star files compile to expected IR
3. IR comparison uses JCS-canonical JSON
4. Error messages include S-series codes
5. `--json` flag produces machine-readable output
6. Documentation complete for all functions

---

## Risk Mitigations Applied

| Risk | Mitigation |
|------|------------|
| R1: Non-determinism | No random/time/IO; all functions pure |
| R2: API bloat | Minimal 15-20 functions; composable primitives |
| R3: Naming conflicts | Domain prefixes (`audio_`, `texture_`, `mesh_`) |
| R4: Error verbosity | Structured S-series codes; `--json` output |
| R5: Golden brittleness | JCS-canonical comparison; semantic assertions |
| R7: Type safety | Validate at function entry; clear error messages |
