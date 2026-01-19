# RFC-0012: Sprite Assets (Sheets and Animation Clips)

- **Status:** ACCEPTED
- **Author:** SpecCade Team
- **Created:** 2026-01-20
- **Target Version:** SpecCade v1.0

## Summary

This RFC defines two new recipe kinds for 2D sprite asset generation:

1. **`sprite.sheet_v1`** - Deterministic spritesheet/atlas packing with frame metadata
2. **`sprite.animation_v1`** - Animation clip definitions referencing spritesheet frames

These recipes enable LLM-native authoring of 2D game assets with full determinism guarantees (Tier 1).

## Motivation

Game development frequently requires spritesheets for:
- Character animation frames
- UI element atlases
- Particle effect frames
- Tilesets

Current approaches involve manual atlas packing or external tools with non-deterministic output. SpecCade can provide:
- Deterministic packing with byte-identical output
- Machine-readable metadata (frame UVs, pivots, timing)
- Tight integration with the existing texture backend (reuses shelf packer from `texture.trimsheet_v1`)

## Design

### 1. `sprite.sheet_v1` Recipe

Packs sprite frames into an atlas with deterministic shelf packing.

**Key differences from `texture.trimsheet_v1`:**
- Frames have explicit pivot points (for sprite origins)
- Frames are designed for animation sequences (ordered by frame index)
- Metadata includes animation-friendly fields (frame IDs, pivots, pixel dimensions)

#### Spec Structure

```json
{
  "spec_version": 1,
  "asset_id": "player-walk-sheet",
  "asset_type": "texture",
  "license": "CC0-1.0",
  "seed": 42,
  "outputs": [
    { "kind": "primary", "format": "png", "path": "sprites/player_walk.png" },
    { "kind": "metadata", "format": "json", "path": "sprites/player_walk.json" }
  ],
  "recipe": {
    "kind": "sprite.sheet_v1",
    "params": {
      "resolution": [512, 512],
      "padding": 2,
      "frames": [
        {
          "id": "idle_0",
          "width": 64,
          "height": 64,
          "pivot": [0.5, 0.0],
          "color": [0.5, 0.5, 0.5, 1.0]
        },
        {
          "id": "idle_1",
          "width": 64,
          "height": 64,
          "pivot": [0.5, 0.0],
          "color": [0.6, 0.6, 0.6, 1.0]
        }
      ]
    }
  }
}
```

#### Parameters

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `resolution` | `[u32, u32]` | Yes | - | Atlas dimensions [width, height] in pixels |
| `padding` | `u32` | No | `2` | Gutter/padding between frames (mip-safe) |
| `frames` | `Frame[]` | Yes | - | List of frame definitions |

#### Frame Definition

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `id` | `string` | Yes | - | Unique frame identifier |
| `width` | `u32` | Yes | - | Frame width in pixels |
| `height` | `u32` | Yes | - | Frame height in pixels |
| `pivot` | `[f64, f64]` | No | `[0.5, 0.5]` | Pivot point in normalized coords (0-1). [0,0] = top-left, [1,1] = bottom-right |
| `color` | `[f64, f64, f64, f64]` | Yes (v1) | - | RGBA fill color (v1 uses solid colors only) |

#### Packing Algorithm

The same deterministic shelf packing algorithm from `texture.trimsheet_v1` is reused:

1. Sort frames by height (descending), then width (descending), then ID (ascending)
2. Place frames on shelves, creating new shelves as needed
3. Apply mip-safe gutter replication at frame edges

This ensures byte-identical output for the same input.

#### Metadata Output

```json
{
  "atlas_width": 512,
  "atlas_height": 512,
  "padding": 2,
  "frames": [
    {
      "id": "idle_0",
      "u_min": 0.00390625,
      "v_min": 0.00390625,
      "u_max": 0.12890625,
      "v_max": 0.12890625,
      "width": 64,
      "height": 64,
      "pivot": [0.5, 0.0]
    }
  ]
}
```

### 2. `sprite.animation_v1` Recipe

Defines animation clips that reference frames from a spritesheet.

**Note:** This recipe produces JSON metadata only (no PNG). It references frames by ID from an associated spritesheet.

#### Spec Structure

```json
{
  "spec_version": 1,
  "asset_id": "player-idle-anim",
  "asset_type": "animation_2d",
  "license": "CC0-1.0",
  "seed": 42,
  "outputs": [
    { "kind": "primary", "format": "json", "path": "anims/player_idle.json" }
  ],
  "recipe": {
    "kind": "sprite.animation_v1",
    "params": {
      "name": "idle",
      "fps": 12,
      "loop_mode": "loop",
      "frames": [
        { "frame_id": "idle_0", "duration_ms": 100 },
        { "frame_id": "idle_1", "duration_ms": 100 },
        { "frame_id": "idle_2", "duration_ms": 100 },
        { "frame_id": "idle_3", "duration_ms": 200 }
      ]
    }
  }
}
```

#### Parameters

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `name` | `string` | Yes | - | Animation clip name |
| `fps` | `u32` | No | `12` | Default frames per second |
| `loop_mode` | `LoopMode` | No | `"loop"` | Playback behavior |
| `frames` | `AnimFrame[]` | Yes | - | Ordered list of animation frames |

#### Loop Mode

| Value | Description |
|-------|-------------|
| `"loop"` | Repeat from start after last frame |
| `"once"` | Play once and stop on last frame |
| `"ping_pong"` | Alternate forward/backward |

#### Animation Frame

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `frame_id` | `string` | Yes | - | Reference to frame ID in spritesheet |
| `duration_ms` | `u32` | No | `1000/fps` | Frame display duration in milliseconds |

#### Metadata Output

```json
{
  "name": "idle",
  "fps": 12,
  "loop_mode": "loop",
  "total_duration_ms": 500,
  "frames": [
    { "frame_id": "idle_0", "duration_ms": 100 },
    { "frame_id": "idle_1", "duration_ms": 100 },
    { "frame_id": "idle_2", "duration_ms": 100 },
    { "frame_id": "idle_3", "duration_ms": 200 }
  ]
}
```

## Determinism Guarantees

Both recipes are **Tier 1** (byte-identical output):

| Aspect | Guarantee |
|--------|-----------|
| Packing order | Deterministic (sorted by height/width/id) |
| UV coordinates | Exact f64 values derived from integer pixel positions |
| PNG encoding | Uses same deterministic encoder as `texture.trimsheet_v1` |
| JSON output | Canonical formatting (sorted keys, 2-space indent) |

## Asset Type Compatibility

| Recipe | Asset Type |
|--------|------------|
| `sprite.sheet_v1` | `texture` |
| `sprite.animation_v1` | `animation_2d` (new) |

**Note:** `animation_2d` is a new asset type for 2D animation metadata. It does not require Blender and produces JSON only.

## Implementation Notes

### Backend Reuse

The `sprite.sheet_v1` backend reuses:
- `speccade_backend_texture::trimsheet` shelf packing algorithm
- `speccade_backend_texture::png` deterministic encoder
- `speccade_backend_texture::maps::TextureBuffer` for rendering

### Frame Content (v1 Scope)

For v1, frames are solid colors only (like trimsheet tiles). Future extensions may add:
- Procedural patterns per frame
- External image references
- Gradient fills

### Validation Rules

| Rule | Error Code | Description |
|------|------------|-------------|
| Frame ID uniqueness | E016 | Duplicate frame IDs in spritesheet |
| Frame fits atlas | E017 | Frame + padding exceeds atlas dimensions |
| Valid pivot range | E018 | Pivot values must be in [0, 1] |
| Valid loop mode | E019 | Unknown loop mode value |
| Non-empty frames | E020 | Animation must have at least one frame |

## Future Extensions

1. **Procedural frame content** - Reference texture graph nodes per frame
2. **Animation events** - Trigger points (footstep, attack, etc.)
3. **Blend shapes** - Morph between frame states
4. **Nested animations** - Animation state machines referencing clips

## Example Workflow

1. Create a spritesheet spec with all character frames
2. Create animation clip specs referencing frame IDs
3. Generate both with `speccade generate`
4. Load metadata JSON in game engine for runtime playback

```bash
speccade generate --spec player_sheet.spec.json --out-root ./assets
speccade generate --spec player_idle.spec.json --out-root ./assets
```

## References

- `texture.trimsheet_v1` - Existing atlas packing implementation
- `docs/rfcs/RFC-0001-canonical-spec.md` - Spec format reference
- `crates/speccade-backend-texture/src/trimsheet.rs` - Shelf packing algorithm
