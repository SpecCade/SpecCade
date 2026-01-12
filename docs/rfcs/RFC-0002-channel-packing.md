# RFC-0002: Unopinionated Channel Packing for Textures

- **Status:** Implemented
- **Author:** SpecCade Team
- **Created:** 2026-01-10
- **Target Version:** SpecCade v1.x
- **Dependencies:** RFC-0001 (Canonical Spec Architecture)
- **Last reviewed:** 2026-01-12

## Summary

This RFC defines a channel packing system for textures that allows spec authors to combine multiple grayscale maps into a single output texture with user-defined channel assignments. The system is intentionally unopinionated about specific packing conventions (ORM, MRE, etc.), instead providing a flexible mechanism that supports any channel layout.

**Design principles:**

- **Unopinionated:** No built-in assumptions about specific packing conventions
- **Flexible:** Authors specify exactly which map goes to which channel
- **Composable:** Works with any combination of generated grayscale maps
- **Engine-agnostic:** Supports Unity, Unreal, Godot, and custom pipelines

---

## 1. Motivation

Different game engines and rendering pipelines use different channel packing conventions:

| Convention | R | G | B | A | Used By |
|------------|---|---|---|---|---------|
| ORM | Occlusion | Roughness | Metallic | - | Unity HDRP |
| MRE | Metallic | Roughness | Emissive | - | Custom |
| ARM | AO | Roughness | Metallic | - | Unreal |
| Smoothness | Smoothness | Smoothness | Smoothness | - | Unity Standard (inverted roughness) |

Rather than hardcoding support for specific conventions, SpecCade provides primitives that let authors define their own packing layouts.

---

## 2. Schema Design

### 2.1 Recipe Kind

A new recipe kind `texture.packed_v1` handles channel-packed texture generation:

```json
{
  "recipe": {
    "kind": "texture.packed_v1",
    "params": {
      "resolution": [512, 512],
      "tileable": true,
      "maps": { ... }
    }
  }
}
```

### 2.2 Map Definitions

The `maps` object defines named grayscale source maps:

```json
{
  "maps": {
    "height": { "type": "pattern", "pattern": "noise", "noise_type": "fbm", "octaves": 4 },
    "ao": { "type": "grayscale", "from_height": true, "ao_strength": 0.5 },
    "roughness": { "type": "grayscale", "from_height": true },
    "metallic": { "type": "grayscale", "value": 1.0 },
    "base": { "type": "pattern", "pattern": "noise", "noise_type": "fbm", "octaves": 4 }
  }
}
```

**Map types:**

| Type | Description | Parameters |
|------|-------------|------------|
| `grayscale` | Solid or derived grayscale | `value`, `from_height`, `ao_strength` |
| `pattern` | Procedural pattern (noise only in v1) | `pattern`, `noise_type`, `octaves` |

**Grayscale parameters:**

- `value` (float 0.0-1.0): Constant grayscale value
- `from_height` (bool): Derive from the shared `height` map
- `ao_strength` (float): Ambient occlusion computation strength (only valid with `from_height`)

**`from_height` behavior:**

- If `from_height: true` **and** `ao_strength` is set, the backend generates AO from the height map
  (strength is clamped to `[0, 1]`).
- Otherwise, the raw height map values (0..1) are used as the grayscale output.

**Height source requirement:**

- If any map uses `from_height: true`, the `maps` object **must** include a `height` key.
- The `height` map is the shared source for all `from_height` maps.

**Pattern maps (v1):**

- `pattern` is restricted to `"noise"`.
- `noise_type` must be one of: `"perlin"`, `"simplex"`, `"fbm"`, `"worley"`.
- `octaves` is only valid when `noise_type: "fbm"`.

### 2.3 Output Channel Specification

The `outputs` array declares packed textures with explicit channel mappings:

```json
{
  "recipe": {
    "kind": "texture.packed_v1",
    "params": {
      "resolution": [256, 256],
      "tileable": true,
      "maps": {
        "height": { "type": "pattern", "pattern": "noise", "noise_type": "fbm", "octaves": 4 },
        "rough": { "type": "grayscale", "from_height": true }
      }
    }
  },
  "outputs": [
    {
      "kind": "packed",
      "format": "png",
      "path": "orm.png",
      "channels": {
        "r": "ao",
        "g": "roughness",
        "b": "metallic"
      }
    }
  ]
}
```

**Channel reference formats:**

1. **Simple string** - Reference map by name:
   ```json
   { "r": "roughness" }
   ```

2. **Object with options** - Reference with options:
   ```json
   { "r": { "key": "roughness", "invert": true, "component": "luminance" } }
   ```

3. **Constant** - Fill a channel with a constant:
   ```json
   { "a": { "constant": 1.0 } }
   ```

**Channel options (extended reference):**

| Option | Type | Description | Default |
|--------|------|-------------|---------|
| `key` | string | Map name to reference | required |
| `component` | string | Extract `r`/`g`/`b`/`a`/`luminance` from RGB(A) sources | `luminance` |
| `invert` | bool | Invert values (1.0 - value) | `false` |

### 2.4 Complete Example: ORM Texture

```json
{
  "spec_version": 1,
  "asset_id": "packed_orm",
  "asset_type": "texture",
  "license": "CC0-1.0",
  "seed": 54322,
  "description": "Packed ORM texture (ambient occlusion, roughness, metallic)",
  "outputs": [
    {
      "kind": "packed",
      "format": "png",
      "path": "orm.png",
      "channels": {
        "r": "ao",
        "g": "roughness",
        "b": "metallic"
      }
    }
  ],
  "recipe": {
    "kind": "texture.packed_v1",
    "params": {
      "resolution": [512, 512],
      "tileable": true,
      "maps": {
        "height": { "type": "pattern", "pattern": "noise", "noise_type": "fbm", "octaves": 4 },
        "ao": { "type": "grayscale", "from_height": true, "ao_strength": 0.5 },
        "roughness": { "type": "grayscale", "from_height": true },
        "metallic": { "type": "grayscale", "value": 1.0 }
      }
    }
  }
}
```

---

## 3. Channel Inversion (Roughness to Smoothness)

A common use case is converting roughness to smoothness (Unity Standard shader). The `invert` option handles this:

```json
{
  "outputs": [
    {
      "kind": "packed",
      "format": "png",
      "path": "smoothness.png",
      "channels": {
        "r": { "key": "rough", "invert": true },
        "g": { "key": "rough", "invert": true },
        "b": { "key": "rough", "invert": true }
      }
    }
  ]
}
```

The backend computes: `output_value = 1.0 - input_value`

---

## 4. Implementation Notes

### 4.1 Backend Responsibilities (Worker P2)

The texture backend must:

1. Parse `texture.packed_v1` recipe
2. Generate all source maps defined in `params.maps`
3. For each output with `kind: "packed"`:
   - Create output buffer at specified resolution
   - For each channel (r, g, b, a):
     - Look up referenced map
     - Apply transforms (invert, component extraction, constants)
     - Write to channel

### 4.2 Spec Type Responsibilities (Worker P1)

The spec types must:

1. Add `PackedOutput` variant to output kinds
2. Define `ChannelRef` enum (string | object with options)
3. Add validation for channel references
4. Ensure referenced maps exist in recipe

### 4.3 Validation Rules

| Rule | Error Code | Description |
|------|------------|-------------|
| Channel references valid map | E020 | Unknown map key in channel reference |
| Packed output must have channels | E021 | Packed output has no channels mapping |
| Packed output must be PNG | E022 | Packed output format is not `png` |
| Packed recipe must declare packed outputs | E023 | No `kind: "packed"` outputs declared |
| `channels` only valid for packed outputs | E015 | `channels` present on non-`packed` output |

---

## 5. Future Extensions

### 5.1 Alpha Channel Support

The current schema supports alpha channel:

```json
{
  "channels": {
    "r": "roughness",
    "g": "metallic",
    "b": "ao",
    "a": "height"
  }
}
```

### 5.2 Additional Transforms

Future versions may add:

- `gamma`: Gamma correction
- `remap`: Value remapping with control points
- `blend`: Blend multiple maps

### 5.3 Multi-Output Generation

A single spec can output multiple packed textures:

```json
{
  "outputs": [
    { "kind": "packed", "path": "orm.png", "channels": { "r": "ao", "g": "rough", "b": "metal" } },
    { "kind": "packed", "path": "emissive.png", "channels": { "r": "glow", "g": "glow", "b": "glow" } }
  ]
}
```

---

## 6. Golden Specs

Reference implementations are provided in the golden corpus:

| File | Description |
|------|-------------|
| `golden/speccade/specs/texture/packed_mre.json` | MRE packing (Metallic-Roughness-Emissive) |
| `golden/speccade/specs/texture/packed_orm.json` | ORM packing (Occlusion-Roughness-Metallic) |
| `golden/speccade/specs/texture/packed_smoothness.json` | Inversion example (roughness to smoothness) |

---

## 7. References

- RFC-0001: Canonical Spec Architecture
- Unity HDRP Mask Map: https://docs.unity3d.com/Packages/com.unity.render-pipelines.high-definition@latest
- Unreal Engine Material Properties: https://docs.unrealengine.com/
