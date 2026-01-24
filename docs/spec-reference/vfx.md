# VFX Asset Recipes

VFX (Visual Effects) recipes generate flipbook-style animations and particle rendering profiles for game effects like explosions, smoke, particles, and energy effects.

## Recipe Kinds

### `vfx.particle_profile_v1`

Generates metadata describing particle rendering profiles for VFX systems. This is a metadata-only recipe (no texture generation) that outputs JSON with blend mode, color grading, and distortion parameters for particle effects.

**Parameters:**

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `profile` | `string` | Yes | - | Profile type: `additive`, `soft`, `distort`, `multiply`, `screen`, `normal` |
| `color_tint` | `[f64, f64, f64]` | No | `[1.0, 1.0, 1.0]` | RGB tint color (each component in [0.0, 1.0]) |
| `intensity` | `f64` | No | `1.0` | Intensity multiplier (must be non-negative) |
| `distortion_strength` | `f64` | No | `0.0` | Distortion strength for `distort` profile (in [0.0, 1.0]) |

**Profile Types:**

- `additive` - Additive blending (bright, glowing effects like fire, sparks, magic)
- `soft` - Soft/premultiplied alpha (smoke, fog, soft particles)
- `distort` - Distortion/refraction effect (heat haze, shockwaves, underwater)
- `multiply` - Multiply blending (shadows, darkening effects)
- `screen` - Screen blending (bright overlay, lightning, lens flares)
- `normal` - Normal alpha blending (standard transparent particles)

**Outputs:**

- **Primary (JSON)**: Particle rendering profile metadata

**Metadata Structure:**

```json
{
  "profile": "additive",
  "blend_mode": "additive",
  "tint": [1.0, 0.6, 0.2],
  "intensity": 1.5,
  "distortion_strength": 0.0,
  "shader_hints": {
    "depth_write": false,
    "transparent": true,
    "soft_particles": false,
    "distortion_pass": false
  }
}
```

**Example Spec:**

```json
{
  "spec_version": 1,
  "asset_id": "vfx-fire-particles",
  "asset_type": "vfx",
  "license": "CC0-1.0",
  "description": "Fire particle rendering profile with warm orange tint",
  "seed": 42,
  "outputs": [
    {
      "kind": "primary",
      "format": "json",
      "path": "vfx/fire_particles.json"
    }
  ],
  "recipe": {
    "kind": "vfx.particle_profile_v1",
    "params": {
      "profile": "additive",
      "color_tint": [1.0, 0.6, 0.2],
      "intensity": 1.5
    }
  }
}
```

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

### `vfx.particle_profile_v1`

Particle profile generation is **Tier 1** (metadata-only, fully deterministic):

- Same params = identical JSON output (seed-independent)
- No randomization or non-deterministic operations
- Pure metadata transformation

### `vfx.flipbook_v1`

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
