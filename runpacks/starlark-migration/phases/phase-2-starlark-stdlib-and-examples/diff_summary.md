# Phase 2 Diff Summary

## Files Created

| File | Description | Lines |
|------|-------------|-------|
| `crates/speccade-cli/src/compiler/stdlib/mod.rs` | stdlib main module with registration | ~90 |
| `crates/speccade-cli/src/compiler/stdlib/core.rs` | spec(), output() functions | ~200 |
| `crates/speccade-cli/src/compiler/stdlib/audio.rs` | Audio synthesis functions | ~550 |
| `crates/speccade-cli/src/compiler/stdlib/texture.rs` | Texture graph functions | ~290 |
| `crates/speccade-cli/src/compiler/stdlib/mesh.rs` | Mesh primitive/modifier functions | ~230 |
| `crates/speccade-cli/src/compiler/stdlib/validation.rs` | Shared validation utilities | ~130 |
| `golden/starlark/audio_synth_oscillator.star` | Audio oscillator example | ~25 |
| `golden/starlark/audio_synth_fm.star` | FM synthesis example | ~25 |
| `golden/starlark/audio_synth_layered.star` | Layered audio example | ~45 |
| `golden/starlark/texture_noise.star` | Noise texture example | ~20 |
| `golden/starlark/texture_colored.star` | Colored texture example | ~25 |
| `golden/starlark/mesh_cube.star` | Mesh cube example | ~20 |
| `golden/starlark/mesh_sphere.star` | Mesh sphere example | ~15 |
| `docs/stdlib-reference.md` | Complete function reference | ~350 |
| `docs/starlark-authoring.md` | Authoring guide | ~270 |
| `runpacks/.../implementation_log.md` | This phase's implementation log | ~150 |
| `runpacks/.../diff_summary.md` | This file | ~100 |

**Total new files: 17**

## Files Modified

| File | Changes |
|------|---------|
| `crates/speccade-cli/src/compiler/mod.rs` | Added `pub mod stdlib` |
| `crates/speccade-cli/src/compiler/eval.rs` | Import stdlib, use GlobalsBuilder, added stdlib test |
| `crates/speccade-cli/src/compiler/error.rs` | Added S-series error codes and stdlib error variants |
| `crates/speccade-tests/tests/starlark_input.rs` | Added 7 stdlib example tests |

## Detailed Changes

### `crates/speccade-cli/src/compiler/mod.rs`

```diff
 mod convert;
 mod error;
 mod eval;
+pub mod stdlib;
```

### `crates/speccade-cli/src/compiler/eval.rs`

```diff
 use super::convert::starlark_to_json;
 use super::error::CompileError;
+use super::stdlib::register_stdlib;
 use super::{CompileResult, CompilerConfig};
 use speccade_spec::Spec;
-use starlark::environment::{Globals, Module};
+use starlark::environment::{GlobalsBuilder, Module};
...
-    let globals = Globals::standard();
+    let globals = GlobalsBuilder::standard()
+        .with(register_stdlib)
+        .build();
```

### `crates/speccade-cli/src/compiler/error.rs`

Added:
- S-series error code documentation
- S001-S006 codes for compiler errors
- S101-S104 codes for stdlib errors
- `StdlibArgument`, `StdlibType`, `StdlibRange`, `StdlibEnum` variants
- `code()` and `category()` methods

### `crates/speccade-tests/tests/starlark_input.rs`

Added tests:
- `load_stdlib_audio_oscillator()`
- `load_stdlib_audio_fm()`
- `load_stdlib_audio_layered()`
- `load_stdlib_texture_noise()`
- `load_stdlib_texture_colored()`
- `load_stdlib_mesh_cube()`
- `load_stdlib_mesh_sphere()`

## New Functions

### Core
| Function | Signature |
|----------|-----------|
| `spec()` | `(asset_id, asset_type, seed, outputs, [recipe], [description], [tags], [license])` |
| `output()` | `(path, format, [kind])` |

### Audio
| Function | Signature |
|----------|-----------|
| `envelope()` | `([attack], [decay], [sustain], [release])` |
| `oscillator()` | `(frequency, [waveform], [sweep_to], [curve])` |
| `fm_synth()` | `(carrier, modulator, index, [sweep_to])` |
| `noise_burst()` | `([noise_type], [filter])` |
| `karplus_strong()` | `(frequency, [decay], [blend])` |
| `lowpass()` | `(cutoff, [resonance], [sweep_to])` |
| `highpass()` | `(cutoff, [resonance], [sweep_to])` |
| `audio_layer()` | `(synthesis, [envelope], [volume], [pan], [filter], [lfo], [delay])` |
| `reverb()` | `([decay], [wet], [room_size])` |
| `delay()` | `([time_ms], [feedback], [wet])` |
| `compressor()` | `([threshold_db], [ratio], [attack_ms], [release_ms])` |

### Texture
| Function | Signature |
|----------|-----------|
| `noise_node()` | `(id, [algorithm], [scale], [octaves], [persistence], [lacunarity])` |
| `gradient_node()` | `(id, [direction], [start], [end])` |
| `constant_node()` | `(id, value)` |
| `threshold_node()` | `(id, input, [threshold])` |
| `invert_node()` | `(id, input)` |
| `color_ramp_node()` | `(id, input, ramp)` |
| `texture_graph()` | `(resolution, nodes, [tileable])` |

### Mesh
| Function | Signature |
|----------|-----------|
| `mesh_primitive()` | `(primitive, dimensions)` |
| `bevel_modifier()` | `([width], [segments])` |
| `subdivision_modifier()` | `([levels], [render_levels])` |
| `decimate_modifier()` | `([ratio])` |
| `mesh_recipe()` | `(primitive, dimensions, [modifiers])` |

**Total new functions: 25**
