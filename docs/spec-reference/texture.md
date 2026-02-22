# Texture Spec Reference

This document covers texture generation in SpecCade.

## Overview

**Asset Type:** `texture`  
**Recipe Kind:** `texture.procedural_v1`  
**Output Format:** PNG

`texture.procedural_v1` is a deterministic, named-node DAG. Each node produces either:

- **Grayscale** (single channel, normalized `[0, 1]`)
- **Color** (RGBA, normalized `[0, 1]`)

Outputs bind file paths to node ids. There are no dedicated "material", "normal", or "packed" recipe kinds.

## Outputs

- Each `primary` output must have `format: "png"`.
- Each `primary` output must set `source` to a node id.
- Grayscale node -> grayscale PNG.
- Color node -> RGBA PNG.

"Packed" textures are just RGBA nodes constructed via `compose_rgba` and written as PNGs.

Example:

```json
{
  "outputs": [
    { "kind": "primary", "format": "png", "path": "albedo.png", "source": "albedo" },
    { "kind": "primary", "format": "png", "path": "orm.png", "source": "packed_orm" }
  ]
}
```

## Params (`recipe.params`)

| Param | Type | Required | Notes |
|------:|------|:--------:|------|
| `resolution` | [integer, integer] | yes | `[width, height]` in pixels |
| `tileable` | boolean | yes | If true, tileable ops (notably `noise`) must wrap seamlessly |
| `nodes` | array | yes | DAG nodes (see below) |

## Node Model

Each node has a stable `id` and a `type` describing its operation:

```json
{ "id": "n", "type": "noise", "noise": { "algorithm": "perlin", "scale": 0.08 } }
```

Nodes reference other nodes by id. The graph must be a DAG (no cycles).

### Grayscale Primitives

- `constant { value }`
- `noise { noise }`
- `reaction_diffusion { steps?, feed?, kill?, diffuse_a?, diffuse_b?, dt?, seed_density? }`
- `gradient { direction, start?, end?, center?, inner?, outer? }`
- `stripes { direction, stripe_width, color1, color2 }`
- `checkerboard { tile_size, color1, color2 }`

### Grayscale Ops

- `invert { input }`
- `clamp { input, min, max }`
- `add { a, b }`
- `multiply { a, b }`
- `lerp { a, b, t }`
- `threshold { input, threshold }`

### Color Ops

- `to_grayscale { input }`
- `color_ramp { input, ramp: ["#RRGGBB", ...] }`
- `palette { input, palette: ["#RRGGBB", ...] }`
- `compose_rgba { r, g, b, a? }`
- `normal_from_height { input, strength }`

### NoiseConfig

`noise` uses the shared `NoiseConfig` shape:

```json
{
  "algorithm": "perlin",
  "scale": 0.05,
  "octaves": 4,
  "persistence": 0.5,
  "lacunarity": 2.0
}
```

Algorithms:

- `perlin`
- `simplex`
- `worley`
- `value`
- `gabor`
- `fbm`

## Example: Minimal Procedural Spec

```json
{
  "spec_version": 1,
  "asset_id": "noise-mask-01",
  "asset_type": "texture",
  "license": "CC0-1.0",
  "seed": 42,
  "outputs": [
    { "kind": "primary", "format": "png", "path": "textures/mask.png", "source": "mask" }
  ],
  "recipe": {
    "kind": "texture.procedural_v1",
    "params": {
      "resolution": [256, 256],
      "tileable": true,
      "nodes": [
        { "id": "n", "type": "noise", "noise": { "algorithm": "perlin", "scale": 0.08 } },
        { "id": "mask", "type": "threshold", "input": "n", "threshold": 0.55 }
      ]
    }
  }
}
```

## Templates (Texture Kits)

SpecCade ships curated procedural texture templates under:

```
packs/preset_library_v1/texture
```

Use the CLI to discover and copy them:

```bash
speccade template list --asset-type texture
speccade template show preset_texture_material_set_basic
speccade template copy preset_texture_material_set_basic --to ./specs/texture/my_material.json
```

Templates are normal `texture.procedural_v1` specs intended as starting points.
