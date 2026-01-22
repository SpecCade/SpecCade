# VFX Asset Recipes

VFX (Visual Effects) recipes generate flipbook-style animations for game effects like explosions, smoke, particles, and energy effects.

## Recipe Kinds

### `vfx.flipbook_v1`

Generates a flipbook animation atlas with procedurally generated frames.

**Parameters:**

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `resolution` | `[u32, u32]` | Yes | - | Atlas resolution [width, height] in pixels |
| `padding` | `u32` | No | `2` | Padding/gutter between frames (for mip-safe borders) |
| `effect` | `string` | Yes | - | Effect type: `explosion`, `smoke`, `energy`, `dissolve` |
| `frame_count` | `u32` | No | `16` | Number of frames in the animation sequence |
| `frame_size` | `[u32, u32]` | Yes | - | Frame dimensions [width, height] in pixels |
| `fps` | `u32` | No | `24` | Animation playback speed in frames per second |
| `loop_mode` | `string` | No | `once` | Playback loop mode: `once`, `loop`, `ping_pong` |

**Effect Types:**

- `explosion` - Expanding radial explosion with fire colors and noise distortion
- `smoke` - Rising turbulent smoke with grayscale colors
- `energy` - Expanding magic circle/energy ring with cyan/blue colors
- `dissolve` - Particle dissolve/fade-out effect

**Outputs:**

- **Primary (PNG)**: Atlas texture containing all packed frames
- **Metadata (JSON)**: Frame metadata with UV coordinates, dimensions, and animation parameters

**Metadata Structure:**

```json
{
  "atlas_width": 512,
  "atlas_height": 512,
  "padding": 2,
  "effect": "explosion",
  "frame_count": 16,
  "frame_size": [64, 64],
  "fps": 24,
  "loop_mode": "once",
  "total_duration_ms": 666,
  "frames": [
    {
      "index": 0,
      "u_min": 0.0,
      "v_min": 0.0,
      "u_max": 0.125,
      "v_max": 0.125,
      "width": 64,
      "height": 64
    }
    // ... more frames
  ]
}
```

**Example Spec:**

```json
{
  "spec_version": "1.0",
  "asset_id": "explosion-vfx-01",
  "asset_type": "vfx",
  "license": "CC0-1.0",
  "description": "Explosion effect flipbook animation",
  "seed": 12345,
  "outputs": [
    {
      "kind": "primary",
      "format": "png",
      "path": "vfx/explosion.png"
    },
    {
      "kind": "metadata",
      "format": "json",
      "path": "vfx/explosion.metadata.json"
    }
  ],
  "recipe": {
    "kind": "vfx.flipbook_v1",
    "params": {
      "resolution": [512, 512],
      "padding": 2,
      "effect": "explosion",
      "frame_count": 16,
      "frame_size": [64, 64],
      "fps": 24,
      "loop_mode": "once"
    }
  }
}
```

## Determinism

VFX flipbook generation is **Tier 1** (fully deterministic):

- Same spec + same seed = byte-identical PNG output
- Frame generation uses deterministic noise (SimplexNoise, FBM)
- Shelf packing algorithm is deterministic (all frames same size)
- PNG encoding uses fixed compression settings

## Usage Guidelines

**Frame Count & Size:**
- Keep `frame_count` * `frame_size` within atlas `resolution`
- Use power-of-two atlas resolutions for better GPU compatibility (256, 512, 1024, 2048)
- Typical frame counts: 8-32 frames for most effects

**Padding:**
- Use `padding: 2` or higher to prevent texture bleeding during mipmap generation
- Padding is filled by replicating edge pixels (mip-safe)

**Effect Selection:**
- `explosion`: Best for impact effects, fire bursts, explosions
- `smoke`: Best for smoke plumes, steam, fog effects
- `energy`: Best for magic effects, power-ups, shields
- `dissolve`: Best for dissolve transitions, particle fades

**Performance:**
- Smaller atlas resolutions generate faster
- Larger frame counts increase generation time linearly
- All effects use noise-based generation (CPU-bound)

## See Also

- [Sprite Assets](sprite.md) - For spritesheet packing with custom frame content
- [Texture Assets](texture.md) - For procedural texture generation
- [Asset Types](asset-types.md) - VFX asset type reference
