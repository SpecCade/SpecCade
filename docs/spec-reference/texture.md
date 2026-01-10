# Texture Spec Reference

This document covers all texture-related asset types and recipes in SpecCade.

## Table of Contents

- [Texture 2D](#texture-2d)
  - [Material Maps (material_maps_v1)](#recipe-texture_2dmaterial_maps_v1)
  - [Normal Maps (normal_map_v1)](#recipe-texture_2dnormal_map_v1)
- [Packed Textures](#packed-textures)
  - [Channel Packing (packed_v1)](#recipe-texturepacked_v1)

---

## Texture 2D

**Asset Type:** `texture_2d`
**Output Formats:** PNG

2D texture maps with coherent multi-layer generation for PBR materials.

### Recipe: `texture_2d.material_maps_v1`

Generates coherent PBR material maps (albedo, roughness, metallic, normal, AO, emissive, height).

#### Required Params

| Param | Type | Description |
|-------|------|-------------|
| `base_material` | object | Base material properties |

#### Optional Params

| Param | Type | Description | Default |
|-------|------|-------------|---------|
| `resolution` | array | Width and height `[w, h]` | `[1024, 1024]` |
| `tileable` | boolean | Generate tileable texture | `false` |
| `maps` | array | Maps to generate | `["albedo"]` |
| `layers` | array | Procedural layers | `[]` |
| `palette` | array | Color palette (hex strings) | `[]` |
| `color_ramp` | array | Grayscale to color ramp | `[]` |

#### Base Material

The `base_material` object defines core material properties:

| Field | Type | Description | Required |
|-------|------|-------------|----------|
| `type` | string | Material type: `"metal"`, `"dielectric"`, `"plastic"` | Yes |
| `base_color` | array | RGB color `[r, g, b]` (0.0-1.0) | Yes |
| `roughness_range` | array | Min/max roughness `[min, max]` | Yes |
| `metallic` | float | Metallic value (0.0-1.0) | Yes |

#### Available Maps

The `maps` array specifies which textures to generate:

- `"albedo"` - Base color/diffuse map
- `"roughness"` - Surface roughness map
- `"metallic"` - Metallic mask map
- `"normal"` - Normal/bump map
- `"ao"` - Ambient occlusion map
- `"emissive"` - Emissive/glow map
- `"height"` - Height/displacement map

#### Layer Types

Layers are applied in order to modify the base material:

##### Noise Pattern Layer

```json
{
  "type": "noise_pattern",
  "noise": {
    "algorithm": "perlin",
    "scale": 0.05,
    "octaves": 4,
    "persistence": 0.5,
    "lacunarity": 2.0
  },
  "affects": ["roughness", "height"],
  "strength": 0.4
}
```

**Noise algorithms:** `"perlin"`, `"simplex"`, `"worley"`, `"value"`, `"fbm"`

| Field | Type | Description |
|-------|------|-------------|
| `algorithm` | string | Noise algorithm |
| `scale` | float | Noise frequency/scale |
| `octaves` | integer | Number of noise octaves |
| `persistence` | float | Amplitude scaling per octave |
| `lacunarity` | float | Frequency scaling per octave |

##### Scratches Layer

```json
{
  "type": "scratches",
  "density": 0.15,
  "length_range": [0.1, 0.4],
  "width": 0.002,
  "affects": ["albedo", "roughness", "normal"],
  "strength": 0.5
}
```

| Field | Type | Description |
|-------|------|-------------|
| `density` | float | Scratch density (0.0-1.0) |
| `length_range` | array | Min/max scratch length |
| `width` | float | Scratch width |

##### Edge Wear Layer

```json
{
  "type": "edge_wear",
  "amount": 0.25,
  "affects": ["roughness", "metallic"]
}
```

| Field | Type | Description |
|-------|------|-------------|
| `amount` | float | Wear amount (0.0-1.0) |

##### Dirt Layer

```json
{
  "type": "dirt",
  "density": 0.2,
  "color": [0.3, 0.25, 0.2],
  "affects": ["albedo", "roughness"],
  "strength": 0.35
}
```

| Field | Type | Description |
|-------|------|-------------|
| `density` | float | Dirt density (0.0-1.0) |
| `color` | array | RGB dirt color |

##### Color Variation Layer

```json
{
  "type": "color_variation",
  "hue_range": 15.0,
  "saturation_range": 0.12,
  "value_range": 0.1,
  "noise_scale": 0.05
}
```

| Field | Type | Description |
|-------|------|-------------|
| `hue_range` | float | Hue variation range (degrees) |
| `saturation_range` | float | Saturation variation |
| `value_range` | float | Value/brightness variation |
| `noise_scale` | float | Variation noise scale |

##### Gradient Layer

```json
{
  "type": "gradient",
  "direction": "vertical",
  "start": 0.0,
  "end": 1.0,
  "affects": ["roughness"],
  "strength": 0.2
}
```

Linear gradient (vertical/horizontal):

| Field | Type | Description |
|-------|------|-------------|
| `direction` | string | `"vertical"` or `"horizontal"` |
| `start` | float | Start value (0.0-1.0) |
| `end` | float | End value (0.0-1.0) |

Radial gradient:

```json
{
  "type": "gradient",
  "direction": "radial",
  "center": [0.5, 0.5],
  "inner": 1.0,
  "outer": 0.0,
  "affects": ["emissive"],
  "strength": 0.3
}
```

| Field | Type | Description |
|-------|------|-------------|
| `direction` | string | `"radial"` |
| `center` | array | Center point `[x, y]` (0.0-1.0) |
| `inner` | float | Value at center |
| `outer` | float | Value at edge |

##### Stripes Layer

```json
{
  "type": "stripes",
  "direction": "vertical",
  "stripe_width": 16,
  "color1": 0.4,
  "color2": 0.6,
  "affects": ["albedo", "roughness"],
  "strength": 0.15
}
```

| Field | Type | Description |
|-------|------|-------------|
| `direction` | string | `"vertical"` or `"horizontal"` |
| `stripe_width` | integer | Width in pixels |
| `color1` | float | First stripe value |
| `color2` | float | Second stripe value |

##### Checkerboard Layer

```json
{
  "type": "checkerboard",
  "tile_size": 32,
  "color1": 0.3,
  "color2": 0.7,
  "affects": ["albedo", "normal", "height"],
  "strength": 0.1
}
```

| Field | Type | Description |
|-------|------|-------------|
| `tile_size` | integer | Tile size in pixels |
| `color1` | float | First tile value |
| `color2` | float | Second tile value |

#### Example: Metal Material

```json
{
  "spec_version": 1,
  "asset_id": "brushed_steel",
  "asset_type": "texture_2d",
  "license": "CC0-1.0",
  "seed": 12345,
  "description": "Brushed steel material with scratches and wear",
  "outputs": [
    {"kind": "primary", "format": "png", "path": "brushed_steel_albedo.png"},
    {"kind": "primary", "format": "png", "path": "brushed_steel_roughness.png"},
    {"kind": "primary", "format": "png", "path": "brushed_steel_metallic.png"},
    {"kind": "primary", "format": "png", "path": "brushed_steel_normal.png"}
  ],
  "recipe": {
    "kind": "texture_2d.material_maps_v1",
    "params": {
      "resolution": [1024, 1024],
      "tileable": true,
      "maps": ["albedo", "roughness", "metallic", "normal"],
      "base_material": {
        "type": "metal",
        "base_color": [0.7, 0.7, 0.75],
        "roughness_range": [0.3, 0.6],
        "metallic": 1.0
      },
      "layers": [
        {
          "type": "noise_pattern",
          "noise": {"algorithm": "simplex", "scale": 0.1, "octaves": 4},
          "affects": ["roughness", "normal"],
          "strength": 0.3
        },
        {
          "type": "scratches",
          "density": 0.15,
          "length_range": [0.1, 0.4],
          "width": 0.002,
          "affects": ["albedo", "roughness"],
          "strength": 0.5
        },
        {
          "type": "edge_wear",
          "amount": 0.25,
          "affects": ["roughness", "metallic"]
        }
      ]
    }
  }
}
```

---

### Recipe: `texture_2d.normal_map_v1`

Generates normal maps from parameterized patterns.

#### Required Params

| Param | Type | Description |
|-------|------|-------------|
| `pattern` or `patterns` | object/array | Pattern definition(s) |

#### Optional Params

| Param | Type | Description | Default |
|-------|------|-------------|---------|
| `resolution` | array | Width and height `[w, h]` | `[1024, 1024]` |
| `size` | array | Alias for resolution | `[1024, 1024]` |
| `tileable` | boolean | Generate tileable texture | `false` |
| `method` | string | Generation method | `"from_pattern"` |
| `processing` | object | Post-processing options | `{}` |

#### Pattern Types

##### Bricks Pattern

```json
{
  "type": "bricks",
  "brick_width": 64,
  "brick_height": 32,
  "brick_size": [64, 32],
  "mortar_width": 4,
  "mortar_depth": 0.3,
  "brick_variation": 0.15,
  "seed": 42,
  "weight": 1.0
}
```

| Field | Type | Description |
|-------|------|-------------|
| `brick_width` | integer | Brick width in pixels |
| `brick_height` | integer | Brick height in pixels |
| `brick_size` | array | Alternative: `[width, height]` |
| `mortar_width` | integer | Mortar gap width |
| `mortar_depth` | float | Mortar depth (0.0-1.0) |
| `brick_variation` | float | Height variation between bricks |

##### Tiles Pattern

```json
{
  "type": "tiles",
  "tile_size": 64,
  "gap_width": 4,
  "gap_depth": 0.25,
  "seed": 123,
  "weight": 0.5
}
```

| Field | Type | Description |
|-------|------|-------------|
| `tile_size` | integer | Tile size in pixels |
| `gap_width` | integer | Gap between tiles |
| `gap_depth` | float | Gap depth (0.0-1.0) |

##### Hexagons Pattern

```json
{
  "type": "hexagons",
  "hex_size": 40,
  "gap_width": 3,
  "gap_depth": 0.2,
  "seed": 456,
  "weight": 0.4
}
```

| Field | Type | Description |
|-------|------|-------------|
| `hex_size` | integer | Hexagon size |
| `gap_width` | integer | Gap between hexagons |
| `gap_depth` | float | Gap depth (0.0-1.0) |

##### Noise Pattern

```json
{
  "type": "noise",
  "scale": 0.05,
  "octaves": 4,
  "height_range": [0.0, 1.0],
  "seed": 789,
  "weight": 0.3
}
```

| Field | Type | Description |
|-------|------|-------------|
| `scale` | float | Noise scale/frequency |
| `octaves` | integer | Number of noise octaves |
| `height_range` | array | Min/max height values |

##### Scratches Pattern

```json
{
  "type": "scratches",
  "density": 80,
  "length_range": [15, 60],
  "depth": 0.2,
  "seed": 321,
  "weight": 0.25
}
```

| Field | Type | Description |
|-------|------|-------------|
| `density` | integer | Number of scratches |
| `length_range` | array | Min/max scratch length in pixels |
| `depth` | float | Scratch depth (0.0-1.0) |

##### Rivets Pattern

```json
{
  "type": "rivets",
  "spacing": 48,
  "radius": 5,
  "height": 0.25,
  "seed": 654,
  "weight": 0.35
}
```

| Field | Type | Description |
|-------|------|-------------|
| `spacing` | integer | Space between rivets |
| `radius` | integer | Rivet radius |
| `height` | float | Rivet height (0.0-1.0) |

##### Weave Pattern

```json
{
  "type": "weave",
  "thread_width": 10,
  "gap": 3,
  "depth": 0.18,
  "weight": 0.3
}
```

| Field | Type | Description |
|-------|------|-------------|
| `thread_width` | integer | Thread width |
| `gap` | integer | Gap between threads |
| `depth` | float | Weave depth (0.0-1.0) |

#### Multi-Pattern Mode

Use `patterns` array to blend multiple patterns:

```json
{
  "patterns": [
    {"type": "bricks", "brick_width": 64, "weight": 1.0},
    {"type": "scratches", "density": 80, "weight": 0.25}
  ]
}
```

Patterns are weighted and blended together.

#### Processing Options

```json
{
  "processing": {
    "strength": 1.5,
    "blur": 0.5,
    "invert": false
  }
}
```

| Field | Type | Description | Default |
|-------|------|-------------|---------|
| `strength` | float | Normal intensity multiplier | `1.0` |
| `blur` | float | Blur amount | `0.0` |
| `invert` | boolean | Invert normals | `false` |

#### Example: Brick Wall Normal

```json
{
  "spec_version": 1,
  "asset_id": "brick_wall_normal",
  "asset_type": "texture_2d",
  "license": "CC0-1.0",
  "seed": 99999,
  "outputs": [
    {"kind": "primary", "format": "png", "path": "brick_wall_normal.png"}
  ],
  "recipe": {
    "kind": "texture_2d.normal_map_v1",
    "params": {
      "resolution": [512, 512],
      "tileable": true,
      "pattern": {
        "type": "bricks",
        "brick_width": 64,
        "brick_height": 32,
        "mortar_width": 4,
        "mortar_depth": 0.3,
        "brick_variation": 0.1
      },
      "processing": {
        "strength": 1.0
      }
    }
  }
}
```

---

## Packed Textures

**Asset Type:** `texture`
**Output Formats:** PNG

Channel-packed textures combine multiple grayscale maps into a single output file with user-defined channel assignments.

### Recipe: `texture.packed_v1`

#### Required Params

| Param | Type | Description |
|-------|------|-------------|
| `maps` | object | Named grayscale source maps |

#### Optional Params

| Param | Type | Description | Default |
|-------|------|-------------|---------|
| `resolution` | array | Width and height `[w, h]` | `[1024, 1024]` |
| `tileable` | boolean | Generate tileable texture | `false` |

#### Map Definitions

The `maps` object defines named grayscale sources:

```json
{
  "maps": {
    "ao": {"type": "grayscale", "from_height": true, "ao_strength": 0.5},
    "roughness": {"type": "grayscale", "from_height": true},
    "metallic": {"type": "grayscale", "value": 1.0},
    "base": {"type": "pattern", "pattern": "noise", "noise_type": "fbm"}
  }
}
```

**Map types:**

| Type | Description | Parameters |
|------|-------------|------------|
| `grayscale` | Solid or derived grayscale | `value`, `from_height`, `ao_strength` |
| `pattern` | Procedural pattern | `pattern`, `noise_type`, `octaves` |

**Grayscale parameters:**

| Field | Type | Description |
|-------|------|-------------|
| `value` | float | Constant value (0.0-1.0) |
| `from_height` | boolean | Derive from height/base map |
| `ao_strength` | float | AO computation strength |

#### Output Channel Specification

Outputs use `kind: "packed"` (or `kind: "primary"`) with a `channels` object:

```json
{
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

#### Channel Reference Formats

**Simple string** - Reference map by name:

```json
{"r": "roughness"}
```

**Object with options** - Reference with transforms:

```json
{"r": {"key": "roughness", "invert": true}}
```

**Channel options:**

| Option | Type | Description | Default |
|--------|------|-------------|---------|
| `key` | string | Map name to reference | required |
| `invert` | boolean | Invert values (1.0 - value) | `false` |
| `scale` | float | Multiply values | `1.0` |
| `bias` | float | Add to values (after scale) | `0.0` |

#### Common Packing Conventions

| Convention | R | G | B | A | Used By |
|------------|---|---|---|---|---------|
| ORM | Occlusion | Roughness | Metallic | - | Unity HDRP |
| MRE | Metallic | Roughness | Emissive | - | Custom |
| ARM | AO | Roughness | Metallic | - | Unreal |

#### Example: ORM Texture

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
        "ao": {"type": "grayscale", "from_height": true, "ao_strength": 0.5},
        "roughness": {"type": "grayscale", "from_height": true},
        "metallic": {"type": "grayscale", "value": 1.0}
      }
    }
  }
}
```

#### Example: Roughness to Smoothness Inversion

```json
{
  "outputs": [
    {
      "kind": "packed",
      "format": "png",
      "path": "smoothness.png",
      "channels": {
        "r": {"key": "rough", "invert": true},
        "g": {"key": "rough", "invert": true},
        "b": {"key": "rough", "invert": true}
      }
    }
  ]
}
```

---

## Validation Rules

### Texture-Specific Validation

| Rule | Error Code | Description |
|------|------------|-------------|
| Channel references valid map | E020 | Unknown map key in channel reference |
| At least one channel specified | E021 | Packed output has no channels |
| Channel key is r, g, b, or a | E022 | Invalid channel name |
| Resolution dimensions positive | E023 | Invalid resolution |
| Layer type is known | E024 | Unknown layer type |

---

## Golden Corpus Specs

Reference implementations:

- `golden/speccade/texture_comprehensive.spec.json` - Comprehensive material test
- `golden/speccade/normal_comprehensive.spec.json` - Comprehensive normal map test
- `golden/speccade/specs/texture/packed_orm.json` - ORM packing example
- `golden/speccade/specs/texture/packed_mre.json` - MRE packing example
