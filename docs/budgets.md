# Budget System

SpecCade enforces resource budgets at validation time to prevent expensive operations from being attempted during generation. This document describes the budget system and its configuration.

## Overview

Budget limits are checked during spec validation before any generation begins. If a spec exceeds any budget limit, validation fails with a clear error message.

## Budget Categories

### Audio Budget

| Limit | Default | Description |
|-------|---------|-------------|
| `max_duration_seconds` | 30.0 | Maximum audio duration in seconds |
| `max_layers` | 32 | Maximum number of audio layers |
| `max_samples` | 1,440,000 | Maximum sample count (30s at 48kHz) |
| `allowed_sample_rates` | 22050, 44100, 48000 | Valid sample rates |

### Texture Budget

| Limit | Default | Description |
|-------|---------|-------------|
| `max_dimension` | 4096 | Maximum width or height in pixels |
| `max_pixels` | 16,777,216 | Maximum total pixels (4096 x 4096) |
| `max_graph_nodes` | 256 | Maximum procedural texture graph nodes |
| `max_graph_depth` | 64 | Maximum graph evaluation depth |

### Music Budget

| Limit | Default (XM) | Default (IT) | Description |
|-------|--------------|--------------|-------------|
| `max_channels` | 32 | 64 | Maximum tracker channels |
| `max_patterns` | 256 | 200 | Maximum patterns |
| `max_instruments` | 128 | 99 | Maximum instruments |
| `max_samples` | N/A | 99 | Maximum samples (IT only) |
| `max_pattern_rows` | 256 | N/A | Maximum rows per pattern (XM) |
| `max_compose_recursion` | 64 | 64 | Maximum compose expansion depth |
| `max_cells_per_pattern` | 50,000 | 50,000 | Maximum cells per pattern |

### Mesh Budget

| Limit | Default | Description |
|-------|---------|-------------|
| `max_vertices` | 100,000 | Maximum vertex count |
| `max_faces` | 100,000 | Maximum face/triangle count |
| `max_bones` | 256 | Maximum skeleton bones |

### General Budget

| Limit | Default | Description |
|-------|---------|-------------|
| `starlark_timeout_seconds` | 30 | Maximum Starlark evaluation time |
| `max_spec_size_bytes` | 10 MB | Maximum spec JSON size |

## Budget Profiles

SpecCade provides pre-defined budget profiles for different use cases:

### Default Profile

The standard profile suitable for most desktop/web game assets.

```rust
use speccade_spec::BudgetProfile;

let profile = BudgetProfile::default();
```

### Strict Profile

Reduced limits for build servers or CI environments where resource usage must be controlled.

```rust
let profile = BudgetProfile::strict();
// Audio: max 10s, 16 layers
// Texture: max 2048x2048
// Mesh: max 50k verts/faces
```

### ZX-8bit Profile

Optimized for retro/8-bit style games with very constrained resources.

```rust
let profile = BudgetProfile::zx_8bit();
// Audio: max 5s, 8 layers, 22050 Hz only
// Texture: max 256x256
// Music: max 8 channels
// Mesh: max 10k verts/faces
```

### Nethercore Profile

Optimized for modern sprite-based games with constrained but contemporary resources.

```rust
let profile = BudgetProfile::nethercore();
// Audio: max 30s, 16 layers, 22050 Hz only
// Texture: max 1024x1024
// Music: max 16 channels (XM/IT)
// Mesh: max 25k verts/faces
```

**Use case:** The Nethercore profile is designed for stylized 2D/sprite-based games targeting modern hardware but maintaining retro-inspired constraints. It uses 22050 Hz as the primary sample rate for a balance between quality and file size, while allowing more layers and longer durations than the ZX-8bit profile. Texture limits support crisp sprites and tilesets at 1024x1024, and music channel counts are doubled compared to ZX-8bit for richer compositions.

## Using Budget Profiles

### In Validation

```rust
use speccade_spec::{validate_for_generate_with_budget, BudgetProfile};

// Use default budgets
let result = validate_for_generate(&spec);

// Use a specific profile
let result = validate_for_generate_with_budget(&spec, &BudgetProfile::strict());
```

### Creating Custom Profiles

```rust
use speccade_spec::{BudgetProfile, AudioBudget};

let mut profile = BudgetProfile::new("custom");
profile.audio.max_duration_seconds = 15.0;
profile.audio.max_layers = 16;
profile.texture.max_dimension = 2048;
```

## Budget Errors

When a budget is exceeded, validation fails with a `BudgetError` that includes:

- `category`: Which budget category was exceeded (audio, texture, music, mesh, general)
- `limit`: The name of the limit that was exceeded
- `actual`: The actual value that exceeded the limit
- `maximum`: The maximum allowed value

Example error message:
```
audio budget exceeded: duration_seconds is 60.0, maximum is 30.0
```

## Rationale

Budget enforcement serves several purposes:

1. **Predictable generation time**: Prevents specs that would take minutes or hours to generate
2. **Memory safety**: Prevents OOM conditions from huge textures or audio files
3. **Format compliance**: Ensures tracker music stays within XM/IT format limits
4. **CI/CD friendliness**: Allows build servers to enforce stricter limits

## Implementation Notes

Budget constants are defined in `speccade-spec/src/validation/budgets.rs` and exported at the crate root.

Backend crates should import budget constants from `speccade-spec` rather than defining their own:

```rust
use speccade_spec::{AudioBudget, TextureBudget, MusicBudget, MeshBudget};

fn validate_duration(seconds: f64) -> bool {
    seconds <= AudioBudget::DEFAULT_MAX_DURATION_SECONDS
}
```

This ensures budget limits are consistent across the entire pipeline.
