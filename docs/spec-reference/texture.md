# Texture Spec Reference

This document covers texture generation in SpecCade.

## Overview

**Asset Type:** `texture`  
**Recipe Kinds:** `texture.material_v1`, `texture.normal_v1`, `texture.packed_v1`  
**Output Formats:** PNG

## Common Output Rules

- Texture backends write PNGs.
- For `texture.material_v1`, you must declare at least one `primary` PNG output per requested map.
- For `texture.normal_v1`, you declare exactly one `primary` PNG output.
- For `texture.packed_v1`, you declare one or more `packed` PNG outputs, each with a `channels` mapping.

## Recipe: `texture.material_v1` (Material Maps)

Generates PBR-style material maps (albedo/roughness/metallic/normal/ao/emissive/height).

### Outputs

`speccade generate` maps generated maps to declared outputs using filenames:

- Preferred: name each output path with a map suffix: `*_albedo.png`, `*_normal.png`, etc.
- Fallback: if you declare exactly one output per map, outputs are matched in the same order as `recipe.params.maps`.

Example:

```json
{
  "outputs": [
    { "kind": "primary", "format": "png", "path": "metal_plate_albedo.png" },
    { "kind": "primary", "format": "png", "path": "metal_plate_roughness.png" },
    { "kind": "primary", "format": "png", "path": "metal_plate_metallic.png" },
    { "kind": "primary", "format": "png", "path": "metal_plate_normal.png" }
  ]
}
```

### Params

| Param | Type | Required | Notes |
|------:|------|:--------:|------|
| `resolution` | [integer, integer] | yes | `[width, height]` |
| `tileable` | boolean | yes | Seamless tiling |
| `maps` | array | yes | List of map types (see below) |
| `base_material` | object | no | Base material preset |
| `layers` | array | no | Procedural layers |
| `palette` | array | no | Hex colors for palette remap |
| `color_ramp` | array | no | Hex colors for ramp interpolation |

#### Map Types (`maps[]`)

- `albedo`
- `normal`
- `roughness`
- `metallic`
- `ao`
- `emissive`
- `height`

#### Base Material (`base_material`)

```json
{
  "type": "metal",
  "base_color": [0.7, 0.7, 0.75],
  "roughness_range": [0.2, 0.8],
  "metallic": 0.9
}
```

`type` is one of:

- `metal`
- `wood`
- `stone`
- `fabric`
- `plastic`
- `concrete`
- `brick`
- `procedural`

Additional optional fields:

- `brick_pattern`: `{ "brick_width": 32, "brick_height": 16, "mortar_width": 2, "offset": 0.5 }`
- `normal_params`: `{ "bump_strength": 1.0, "mortar_depth": 0.3 }`

#### Layers (`layers[]`)

Each layer is a tagged union with `type`:

- `noise_pattern`: `{ "type": "noise_pattern", "noise": { ... }, "affects": [...], "strength": 0.5 }`
- `scratches`: `{ "type": "scratches", "density": 0.1, "length_range": [0.1, 0.4], "width": 0.002, "affects": [...], "strength": 0.5 }`
- `edge_wear`: `{ "type": "edge_wear", "amount": 0.4, "affects": [...] }`
- `dirt`: `{ "type": "dirt", "density": 0.2, "color": [0.2, 0.15, 0.1], "affects": [...], "strength": 0.5 }`
- `color_variation`: `{ "type": "color_variation", "hue_range": 10.0, "saturation_range": 0.2, "value_range": 0.2, "noise_scale": 0.05 }`
- `gradient`: `{ "type": "gradient", "direction": "horizontal", "start": 0.0, "end": 1.0, "affects": [...], "strength": 0.5 }`
- `stripes`: `{ "type": "stripes", "direction": "vertical", "stripe_width": 8, "color1": 0.2, "color2": 0.8, "affects": [...], "strength": 0.5 }`
- `checkerboard`: `{ "type": "checkerboard", "tile_size": 16, "color1": 0.2, "color2": 0.8, "affects": [...], "strength": 0.5 }`
- `pitting`: `{ "type": "pitting", "noise": { ... }, "threshold": 0.6, "depth": 0.2, "affects": [...] }`
- `weave`: `{ "type": "weave", "thread_width": 8, "gap": 2, "depth": 0.2, "affects": [...] }`

Noise config (`noise`) is:

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
- `fbm`

## Recipe: `texture.normal_v1` (Normal Map)

Generates a standalone normal map.

### Outputs

- Declare exactly one `primary` output with `format: "png"`.

### Params

| Param | Type | Required | Default |
|------:|------|:--------:|---------|
| `resolution` | [integer, integer] | yes | — |
| `tileable` | boolean | yes | — |
| `pattern` | object | no | omitted |
| `bump_strength` | number | no | `1.0` |
| `processing` | object | no | omitted |

`processing`:

```json
{ "blur": 0.5, "invert": false }
```

`pattern` is a tagged union with `type`:

- `grid`: `{ "type": "grid", "cell_size": 32, "line_width": 2, "bevel": 0.5 }`
- `bricks`: `{ "type": "bricks", "brick_width": 32, "brick_height": 16, "mortar_width": 2, "offset": 0.5 }`
- `hexagons`: `{ "type": "hexagons", "size": 16, "gap": 2 }`
- `noise_bumps`: `{ "type": "noise_bumps", "noise": { ... } }`
- `diamond_plate`: `{ "type": "diamond_plate", "diamond_size": 32, "height": 0.6 }`
- `tiles`: `{ "type": "tiles", "tile_size": 32, "gap_width": 2, "gap_depth": 0.5, "seed": 1 }`
- `rivets`: `{ "type": "rivets", "spacing": 32, "radius": 3, "height": 0.6, "seed": 1 }`
- `weave`: `{ "type": "weave", "thread_width": 6, "gap": 2, "depth": 0.4 }`

## Recipe: `texture.packed_v1` (Channel Packing)

Generates one or more packed RGBA textures by packing generated maps into output channels.

### Outputs

Each packed output must have:

- `kind: "packed"`
- `format: "png"`
- a `channels` mapping

Example:

```json
{
  "kind": "packed",
  "format": "png",
  "path": "pbr_mra.png",
  "channels": {
    "r": "metallic",
    "g": "roughness",
    "b": { "key": "ao", "invert": true },
    "a": { "constant": 1.0 }
  }
}
```

You can declare multiple packed outputs in the same spec (each with its own `channels` mapping).

### Params

| Param | Type | Required |
|------:|------|:--------:|
| `resolution` | [integer, integer] | yes |
| `tileable` | boolean | yes |
| `maps` | object | yes |

`maps` is a map of user-defined keys to `MapDefinition` objects.

### Map Definitions (`maps.*`)

`MapDefinition` is a tagged union with `type`:

- `grayscale` (constant / derived):
  - `{ "type": "grayscale", "value": 0.5 }`
  - `{ "type": "grayscale", "from_height": true }`
  - `{ "type": "grayscale", "from_height": true, "ao_strength": 0.5 }`
- `pattern`:
  - `{ "type": "pattern", "pattern": "noise", "noise_type": "fbm", "octaves": 4 }`

Notes:

- If any map uses `from_height: true`, you must define a `height` map in `maps`.
- `from_height` uses the shared `height` map:
  - If `ao_strength` is set, AO is generated from height.
  - Otherwise the raw height map values are used as the grayscale output.
- `pattern` is restricted to `"noise"` in v1.
- `noise_type` must be `"perlin"`, `"simplex"`, `"fbm"`, or `"worley"`.
- `octaves` is only valid when `noise_type: "fbm"`.

### Channel Sources (`channels`)

Each channel (`r`, `g`, `b`, optional `a`) is a `ChannelSource`:

- **Key reference:** `"my_key"`
- **Extended reference:** `{ "key": "my_key", "component": "luminance", "invert": false }`
- **Constant:** `{ "constant": 0.5 }`

Valid `component` values are: `r`, `g`, `b`, `a`, `luminance`.
