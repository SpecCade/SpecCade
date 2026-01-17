# Phase 2 Followups

**Date**: 2026-01-17
**Phase**: Phase 2 - Starlark stdlib + presets + examples

## Items for Phase 3

### 1. Unused Import Cleanup (Outside stdlib scope)

The following files have unused imports in their test modules:

| File | Line | Issue |
|------|------|-------|
| `crates/speccade-cli/src/input.rs` | 271 | Unused `std::io::Write` import in tests |
| `crates/speccade-cli/src/commands/eval.rs` | 111 | Unused `std::io::Write` import in tests |

**Recommendation**: Remove these imports during Phase 3 or a general cleanup pass.

### 2. Duplicate `extract_float` Function

The `audio.rs` module defines its own local `extract_float()` function (line 732) while `validation.rs` provides a shared version. The local version returns `anyhow::Result` while the shared version returns `Result<f64, String>`.

**Recommendation**: Consider consolidating to use the shared validation function with error conversion, or document the intentional divergence for return type compatibility.

### 3. stdlib Version Tracking

Per SCOPING.md:
> Consider `stdlib_version` in cache keys (Phase 3 may formalize)

**Recommendation**: Add a `STDLIB_VERSION` constant to `mod.rs` for cache key computation. This ensures cache invalidation when stdlib functions change.

### 4. Additional Texture Nodes

The current texture stdlib covers basic operations but may benefit from additional nodes:

- `blend_node()` - Blend two inputs
- `scale_node()` - Rescale values
- `tile_node()` - Tiling control
- `output_node()` - Mark graph output

**Recommendation**: Gather user feedback before expanding the API.

### 5. Audio LFO Support

The `audio_layer()` function accepts an `lfo` parameter but there is no stdlib helper to create LFO configurations.

**Recommendation**: Consider adding `lfo()` helper function for modulation:
```starlark
lfo(target = "cutoff", frequency = 2.0, depth = 0.5, waveform = "sine")
```

### 6. Recipe Kind Helpers

The stdlib provides low-level recipe params but not complete recipe construction. Users must manually specify `kind` and wrap params.

**Recommendation**: Consider adding recipe helpers:
```starlark
audio_recipe(layers, effects = [], duration = 1.0)
texture_recipe(graph_params)
mesh_recipe_blender(primitive_params)
```

## Technical Debt

### 1. Validation Pattern Repetition

Many stdlib functions repeat the pattern:
```rust
validate_xxx(...).map_err(|e| anyhow::anyhow!(e))?;
```

**Recommendation**: Consider a macro or helper to reduce boilerplate:
```rust
validate!(positive, value, "func", "param")?;
```

### 2. Dict Construction Boilerplate

Every function manually creates dicts with `new_dict()` and `insert_hashed()`. This is verbose but type-safe.

**Recommendation**: Consider a builder pattern or macro if the pattern continues to grow:
```rust
dict_builder!(heap)
    .insert("key", value)
    .insert("key2", value2)
    .build()
```

### 3. Test Helper Location

The `eval_to_json()` test helper is in `mod.rs` and shared across all test modules. This works but couples test utilities to the main module.

**Recommendation**: Consider moving test utilities to a dedicated `tests.rs` module if test infrastructure grows.

## Notes

- All stdlib functions are deterministic (no random, time, or IO)
- Golden tests in `speccade-tests` verify output stability
- Error codes are stable and documented for LLM consumption
