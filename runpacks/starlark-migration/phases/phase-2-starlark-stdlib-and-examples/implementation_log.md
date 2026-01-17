# Phase 2 Implementation Log

## Summary

Phase 2 implements the Starlark standard library (stdlib) for SpecCade, providing
helper functions that reduce boilerplate and improve the authoring experience for
Starlark specs.

## Implementation Steps

### Step 1: stdlib Module Structure

Created the stdlib module structure under `crates/speccade-cli/src/compiler/stdlib/`:

- `mod.rs` - Main module with `register_stdlib()` function
- `core.rs` - Core functions: `spec()`, `output()`
- `audio.rs` - Audio synthesis functions
- `texture.rs` - Texture graph functions
- `mesh.rs` - Mesh primitive and modifier functions
- `validation.rs` - Shared validation utilities

The module is wired into the compiler via `GlobalsBuilder::with(register_stdlib)`.

### Step 2: Core Functions

Implemented `spec()` and `output()`:

- `spec()` - Creates complete spec dict with all required fields
- `output()` - Creates output specification dict

Both functions validate their inputs and return structured error messages.

### Step 3: Audio Functions

Implemented synthesis and effect helpers:

- `envelope()` - ADSR envelope
- `oscillator()` - Basic waveform oscillator
- `fm_synth()` - FM synthesis
- `noise_burst()` - Filtered noise
- `karplus_strong()` - Plucked string synthesis
- `lowpass()`, `highpass()` - Filters
- `audio_layer()` - Complete synthesis layer
- `reverb()`, `delay()`, `compressor()` - Effects

### Step 4: Texture Functions

Implemented texture graph node helpers:

- `noise_node()` - Procedural noise generation
- `gradient_node()` - Gradient patterns
- `constant_node()` - Constant values
- `threshold_node()` - Binary threshold
- `invert_node()` - Inversion
- `color_ramp_node()` - Color mapping
- `texture_graph()` - Complete graph params

### Step 5: Mesh Functions

Implemented mesh primitive and modifier helpers:

- `mesh_primitive()` - Base primitive specification
- `bevel_modifier()` - Bevel modifier
- `subdivision_modifier()` - Subdivision surface
- `decimate_modifier()` - Polygon reduction
- `mesh_recipe()` - Complete recipe params

### Step 6: Wire into Compiler

Updated `crates/speccade-cli/src/compiler/`:

- `mod.rs` - Added `pub mod stdlib`
- `eval.rs` - Changed `Globals::standard()` to `GlobalsBuilder::standard().with(register_stdlib).build()`

This makes all stdlib functions available during Starlark evaluation.

### Step 7: S-series Error Codes

Updated `crates/speccade-cli/src/compiler/error.rs`:

- Added S001-S006 codes for compiler errors (syntax, runtime, timeout, etc.)
- Added S101-S104 codes for stdlib errors (argument, type, range, enum)
- Added `code()` and `category()` methods to `CompileError`

### Step 8: Example .star Files

Created example files in `golden/starlark/`:

- `audio_synth_oscillator.star` - Simple oscillator with envelope
- `audio_synth_fm.star` - FM synthesis example
- `audio_synth_layered.star` - Multi-layer with filter sweep
- `texture_noise.star` - Procedural noise texture
- `texture_colored.star` - Colored noise with gradient
- `mesh_cube.star` - Cube with modifiers
- `mesh_sphere.star` - Decimated sphere

### Step 9: Golden Tests

Added tests to `crates/speccade-tests/tests/starlark_input.rs`:

- `load_stdlib_audio_oscillator()` - Tests audio oscillator example
- `load_stdlib_audio_fm()` - Tests FM synthesis example
- `load_stdlib_audio_layered()` - Tests layered audio example
- `load_stdlib_texture_noise()` - Tests noise texture example
- `load_stdlib_texture_colored()` - Tests colored texture example
- `load_stdlib_mesh_cube()` - Tests mesh cube example
- `load_stdlib_mesh_sphere()` - Tests mesh sphere example

### Step 10: Documentation

Created documentation in `docs/`:

- `stdlib-reference.md` - Complete function reference with signatures and examples
- `starlark-authoring.md` - Authoring guide with best practices

## Testing

All stdlib functions include unit tests that verify:

1. Default parameter values
2. Custom parameter values
3. Error conditions (invalid ranges, enum values, etc.)
4. Error message format (S-series codes)

Tests are located in each stdlib module and can be run with:

```bash
cargo test -p speccade-cli
```

Integration tests in `speccade-tests` verify that the example .star files can be
loaded and produce valid specs.

## Design Notes

### Error Messages

Stdlib errors use a consistent format:

```
S103: oscillator(): 'frequency' must be positive, got -440
```

Error messages include:
- Stable error code (S-series)
- Function name
- Parameter name
- What was expected
- What was received

For enum validation, suggestions are provided if a close match exists:

```
S104: oscillator(): 'waveform' must be one of: sine, square, sawtooth, triangle, pulse. Did you mean 'sine'?
```

### Composability

All stdlib functions return Starlark dicts that can be:
- Modified after creation
- Combined with other dicts
- Passed to other functions

This allows for flexible composition patterns:

```starlark
layer = audio_layer(oscillator(440))
layer["volume"] = 0.5  # Modify after creation
```

### Validation

Validation happens at evaluation time, not compilation time. This means errors
are reported when the function is called, with the actual values that were passed.

The validation module (`validation.rs`) provides reusable validators:
- `validate_positive()` - Positive number check
- `validate_unit_range()` - 0.0-1.0 range
- `validate_pan_range()` - -1.0 to 1.0 range
- `validate_enum()` - Enum value with typo suggestions
- `validate_non_empty()` - Non-empty string check

---

## Retry 1: Starlark 0.12.0 API Compatibility Fixes

### Problem

The initial implementation used Starlark APIs that were internal/private in
version 0.12.0, causing 220+ compilation errors:

1. **Missing `ValueLike` trait** - Required for `.to_value()` method on allocated values
2. **Private `alloc_typed_unchecked` method** - Used for Dict construction
3. **Private `alloc_list` method** - Used for list allocation
4. **`starlark::Error` to `anyhow::Error` conversion** - `get_hashed()?` returns starlark::Error

### Solution

#### 1. Added Required Imports

Added to all stdlib files (`audio.rs`, `texture.rs`, `mesh.rs`, `core.rs`):

```rust
use starlark::collections::SmallMap;
use starlark::values::list::AllocList;
use starlark::values::{..., ValueLike};
```

#### 2. Dict Construction Pattern

**Before (broken):**
```rust
let mut dict = Dict::new(heap.alloc_typed_unchecked(Default::default()).as_ref());
```

**After (fixed):**
```rust
fn new_dict<'v>(heap: &'v Heap) -> Dict<'v> {
    let map: SmallMap<starlark::values::Hashed<Value<'v>>, Value<'v>> = SmallMap::new();
    Dict::new(heap.alloc(map))
}

let mut dict = new_dict(heap);
```

#### 3. List Allocation Pattern

**Before (broken):**
```rust
let list = heap.alloc_list(&values);
dict.insert_hashed(key, list.to_value());
```

**After (fixed):**
```rust
let list = heap.alloc(AllocList(values));
dict.insert_hashed(key, list);
```

#### 4. Hash Key Pattern

**Before (broken):**
```rust
dict.insert_hashed(
    heap.alloc_str("key").to_value().get_hashed()?,
    value,
);
```

**After (fixed):**
```rust
fn hashed_key<'v>(heap: &'v Heap, key: &str) -> starlark::values::Hashed<Value<'v>> {
    heap.alloc_str(key)
        .to_value()
        .get_hashed()
        .expect("string hashing cannot fail")
}

dict.insert_hashed(hashed_key(heap, "key"), value);
```

Using `.expect()` instead of `?` is safe because string hashing cannot fail in
Starlark - it's a pure hash computation with no error conditions.

### Files Modified

- `crates/speccade-cli/src/compiler/stdlib/audio.rs`
- `crates/speccade-cli/src/compiler/stdlib/texture.rs`
- `crates/speccade-cli/src/compiler/stdlib/mesh.rs`
- `crates/speccade-cli/src/compiler/stdlib/core.rs`

### Changes Summary

| Pattern | Occurrences Fixed |
|---------|------------------|
| `Dict::new(heap.alloc_typed_unchecked(...))` | 30+ |
| `heap.alloc_list(...)` | 8 |
| `get_hashed()?` | 100+ |
| Missing `ValueLike` import | 4 files |

### Testing Note

The raw string literal tests (e.g., `r#"color_ramp_node(...)"#`) were initially
suspected of causing issues, but they are correctly formed. The `#` characters
inside the string (`#000000`, `#ffffff`) are hex color codes, not raw string
delimiters.

---

## Retry 2: Starlark 0.12.0 Additional API Fixes

### Problem

After Retry 1, 19 compilation errors remained:

1. **Hashed Type Location (8 errors)** - `starlark::values::Hashed` should be `starlark::collections::Hashed`
2. **Dict::new() Signature Mismatch (4 errors)** - `Dict::new()` takes a `SmallMap` directly, not an allocated value
3. **Raw String Literal Parsing (7 errors)** - Rust 2021 interprets `#ffffff` as a reserved prefix in raw strings
4. **Missing ValueLike Import (1 error)** - `mod.rs` was missing the `ValueLike` trait import

### Solution

#### 1. Hashed Type Location

**Before:**
```rust
fn hashed_key<'v>(heap: &'v Heap, key: &str) -> starlark::values::Hashed<Value<'v>> {
```

**After:**
```rust
fn hashed_key<'v>(heap: &'v Heap, key: &str) -> starlark::collections::Hashed<Value<'v>> {
```

The `Hashed` type is in `starlark::collections`, not `starlark::values`.

#### 2. Dict::new() Signature

**Before:**
```rust
fn new_dict<'v>(heap: &'v Heap) -> Dict<'v> {
    let map: SmallMap<starlark::values::Hashed<Value<'v>>, Value<'v>> = SmallMap::new();
    Dict::new(heap.alloc(map))
}
```

**After:**
```rust
fn new_dict<'v>(_heap: &'v Heap) -> Dict<'v> {
    let map: SmallMap<starlark::collections::Hashed<Value<'v>>, Value<'v>> = SmallMap::new();
    Dict::new(map)
}
```

`Dict::new()` takes a `SmallMap` directly, not a heap-allocated version. The `_heap` parameter is kept for signature compatibility but unused.

#### 3. Raw String Literal Parsing

**Before:**
```rust
let result = eval_to_json(r#"color_ramp_node("colored", "noise", ["#000000", "#ffffff"])"#);
```

**After:**
```rust
let result = eval_to_json("color_ramp_node(\"colored\", \"noise\", [\"#000000\", \"#ffffff\"])");
```

Rust 2021 reserves `#` as a prefix in raw strings when followed by certain characters. Using regular escaped strings avoids this issue.

#### 4. Missing ValueLike Import

**Added to `mod.rs`:**
```rust
use starlark::values::ValueLike;
```

### Files Modified

| File | Changes |
|------|---------|
| `crates/speccade-cli/src/compiler/stdlib/audio.rs` | Fixed `Hashed` import path, `Dict::new()` signature |
| `crates/speccade-cli/src/compiler/stdlib/texture.rs` | Fixed `Hashed` import path, `Dict::new()` signature, raw string tests |
| `crates/speccade-cli/src/compiler/stdlib/mesh.rs` | Fixed `Hashed` import path, `Dict::new()` signature |
| `crates/speccade-cli/src/compiler/stdlib/core.rs` | Fixed `Hashed` import path, `Dict::new()` signature |
| `crates/speccade-cli/src/compiler/stdlib/mod.rs` | Added `ValueLike` import |

### Error Count Summary

| Category | Count Fixed |
|----------|-------------|
| `starlark::values::Hashed` -> `starlark::collections::Hashed` | 8 |
| `Dict::new(heap.alloc(map))` -> `Dict::new(map)` | 4 |
| Raw string literal `r#"...#ffffff..."#` | 6 |
| Missing `ValueLike` import | 1 |
| **Total** | **19** |

---

## Retry 3: Dict::new() Final Fix and Unused Function Cleanup

### Problem

After Retry 2, 5 compilation errors remained:

1. **Dict::new() Signature Mismatch (4 errors)** - `Dict::new()` in Starlark 0.12.0 expects `SmallMap<Value, Value>`, NOT `SmallMap<Hashed<Value>, Value>`
2. **Lifetime Error in mod.rs (1 error)** - `eval_with_stdlib()` returns `Value<'static>` but references a local `Module`

### Root Cause Analysis

#### 1. Dict::new() Signature

The Starlark 0.12.0 `Dict::new()` function signature is:

```rust
pub fn new(content: SmallMap<Value<'v>, Value<'v>>) -> Self
```

The previous fix attempted to use `SmallMap<Hashed<Value>, Value>`, which was incorrect. The dict handles hashing internally when you call `insert_hashed()`.

#### 2. eval_with_stdlib() Lifetime Issue

The function attempted to return `Value<'static>` but the `Value` was borrowed from a local `Module` that would be dropped at the end of the function. Since all tests use `eval_to_json()` instead, this function was simply removed.

### Solution

#### 1. Fixed Dict::new() Signature

**Before (incorrect):**
```rust
fn new_dict<'v>(_heap: &'v Heap) -> Dict<'v> {
    let map: SmallMap<starlark::collections::Hashed<Value<'v>>, Value<'v>> = SmallMap::new();
    Dict::new(map)
}
```

**After (correct):**
```rust
fn new_dict<'v>(_heap: &'v Heap) -> Dict<'v> {
    let map: SmallMap<Value<'v>, Value<'v>> = SmallMap::new();
    Dict::new(map)
}
```

#### 2. Removed Unused eval_with_stdlib() Function

The `eval_with_stdlib()` function in `mod.rs` was removed entirely since:
- It had an unfixable lifetime issue (returning `Value<'static>` from local `Module`)
- All tests use `eval_to_json()` instead
- The unused `ValueLike` import was also removed

### Files Modified

| File | Changes |
|------|---------|
| `crates/speccade-cli/src/compiler/stdlib/audio.rs` | Changed `SmallMap<Hashed<Value>, Value>` to `SmallMap<Value, Value>` |
| `crates/speccade-cli/src/compiler/stdlib/texture.rs` | Changed `SmallMap<Hashed<Value>, Value>` to `SmallMap<Value, Value>` |
| `crates/speccade-cli/src/compiler/stdlib/mesh.rs` | Changed `SmallMap<Hashed<Value>, Value>` to `SmallMap<Value, Value>` |
| `crates/speccade-cli/src/compiler/stdlib/core.rs` | Changed `SmallMap<Hashed<Value>, Value>` to `SmallMap<Value, Value>` |
| `crates/speccade-cli/src/compiler/stdlib/mod.rs` | Removed `eval_with_stdlib()` function, removed unused `ValueLike` import |

### Error Count Summary

| Category | Count Fixed |
|----------|-------------|
| `SmallMap<Hashed<Value>, Value>` -> `SmallMap<Value, Value>` | 4 |
| Removed `eval_with_stdlib()` with lifetime error | 1 |
| **Total** | **5** |

### Key Insight

The `insert_hashed()` method on `Dict` takes care of the hashing internally. The `SmallMap` used to construct the dict should use plain `Value` keys, not pre-hashed keys. The `hashed_key()` helper function is still used when calling `insert_hashed()`, but the map itself stores values indexed by `Value`, not `Hashed<Value>`.
