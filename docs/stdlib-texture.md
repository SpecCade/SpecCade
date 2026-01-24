# SpecCade Starlark Standard Library - Texture Functions

[← Back to Index](stdlib-reference.md)

## Texture Functions

Texture functions provide a node-based procedural texture graph system.

## Table of Contents
- [Node Functions](#node-functions)
- [Graph Functions](#graph-functions)
- [Trimsheet Functions](#trimsheet-functions)
- [Decal Functions](#decal-functions)
- [Splat Set Functions](#splat-set-functions)
- [Matcap Functions](#matcap-functions)
- [Material Preset Functions](#material-preset-functions)

---

## Node Functions

### noise_node()

Creates a noise texture graph node.

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| id | str | Yes | - | Unique node identifier |
| algorithm | str | No | "perlin" | "perlin", "simplex", "worley", "value", "fbm" |
| scale | f64 | No | 0.1 | Noise scale factor |
| octaves | int | No | 4 | Number of octaves |
| persistence | f64 | No | 0.5 | Amplitude decay per octave |
| lacunarity | f64 | No | 2.0 | Frequency multiplier per octave |

**Returns:** Dict matching TextureProceduralNode with Noise op.

**Example:**
```python
noise_node("height", "perlin", 0.1, 4)
noise_node("detail", "simplex", 0.05, 6, 0.5, 2.0)
```

### gradient_node()

Creates a gradient texture node.

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| id | str | Yes | - | Unique node identifier |
| direction | str | No | "horizontal" | "horizontal", "vertical", "radial" |
| start | f64 | No | 0.0 | Start value |
| end | f64 | No | 1.0 | End value |
| center | list | No | None | Center point for radial gradient [x, y] |
| inner | f64 | No | None | Inner radius for radial gradient |
| outer | f64 | No | None | Outer radius for radial gradient |

**Returns:** Dict matching TextureProceduralNode with Gradient op.

**Example:**
```python
gradient_node("grad", "horizontal")
gradient_node("vignette", "radial", 1.0, 0.0)
gradient_node("custom_radial", "radial", 1.0, 0.0, [0.5, 0.5], 0.0, 1.0)
```

### constant_node()

Creates a constant value texture node.

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| id | str | Yes | - | Unique node identifier |
| value | f64 | Yes | - | Constant value (0.0-1.0) |

**Returns:** Dict matching TextureProceduralNode with Constant op.

**Example:**
```python
constant_node("white", 1.0)
constant_node("gray", 0.5)
```

### threshold_node()

Creates a threshold operation node.

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| id | str | Yes | - | Unique node identifier |
| input | str | Yes | - | Input node id |
| threshold | f64 | No | 0.5 | Threshold value |

**Returns:** Dict matching TextureProceduralNode with Threshold op.

**Example:**
```python
threshold_node("mask", "noise", 0.5)
```

### invert_node()

Creates an invert operation node (1 - x).

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| id | str | Yes | - | Unique node identifier |
| input | str | Yes | - | Input node id |

**Returns:** Dict matching TextureProceduralNode with Invert op.

**Example:**
```python
invert_node("inverted", "noise")
```

### color_ramp_node()

Creates a color ramp mapping node.

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| id | str | Yes | - | Unique node identifier |
| input | str | Yes | - | Input node id |
| ramp | list | Yes | - | List of hex colors (at least 2) |

**Returns:** Dict matching TextureProceduralNode with ColorRamp op.

**Example:**
```python
color_ramp_node("colored", "noise", ["#000000", "#ff0000", "#ffffff"])
```

### add_node()

Creates an add blend node (a + b).

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| id | str | Yes | - | Unique node identifier |
| a | str | Yes | - | First input node id |
| b | str | Yes | - | Second input node id |

**Returns:** Dict matching TextureProceduralNode with Add op.

**Example:**
```python
add_node("combined", "noise1", "noise2")
```

### multiply_node()

Creates a multiply blend node (a * b).

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| id | str | Yes | - | Unique node identifier |
| a | str | Yes | - | First input node id |
| b | str | Yes | - | Second input node id |

**Returns:** Dict matching TextureProceduralNode with Multiply op.

**Example:**
```python
multiply_node("masked", "noise", "gradient")
```

### lerp_node()

Creates a lerp (linear interpolation) node.

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| id | str | Yes | - | Unique node identifier |
| a | str | Yes | - | First input node id |
| b | str | Yes | - | Second input node id |
| t | str | Yes | - | Interpolation factor node id (0 = a, 1 = b) |

**Returns:** Dict matching TextureProceduralNode with Lerp op.

**Example:**
```python
lerp_node("blended", "noise1", "noise2", "mask")
```

### clamp_node()

Creates a clamp node.

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| id | str | Yes | - | Unique node identifier |
| input | str | Yes | - | Input node id |
| min | f64 | No | 0.0 | Minimum value |
| max | f64 | No | 1.0 | Maximum value |

**Returns:** Dict matching TextureProceduralNode with Clamp op.

**Example:**
```python
clamp_node("clamped", "noise", 0.2, 0.8)
```

### stripes_node()

Creates a stripes pattern node.

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| id | str | Yes | - | Unique node identifier |
| direction | str | Yes | - | Stripe direction: "horizontal" or "vertical" |
| stripe_width | int | Yes | - | Width of each stripe in pixels |
| color1 | f64 | No | 0.0 | First stripe value (0.0-1.0) |
| color2 | f64 | No | 1.0 | Second stripe value (0.0-1.0) |

**Returns:** Dict matching TextureProceduralNode with Stripes op.

**Example:**
```python
stripes_node("lines", "horizontal", 4, 0.0, 1.0)
```

### checkerboard_node()

Creates a checkerboard pattern node.

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| id | str | Yes | - | Unique node identifier |
| tile_size | int | Yes | - | Size of each tile in pixels |
| color1 | f64 | No | 0.0 | First tile value (0.0-1.0) |
| color2 | f64 | No | 1.0 | Second tile value (0.0-1.0) |

**Returns:** Dict matching TextureProceduralNode with Checkerboard op.

**Example:**
```python
checkerboard_node("checker", 8, 0.0, 1.0)
```

### grayscale_node()

Creates a grayscale conversion node.

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| id | str | Yes | - | Unique node identifier |
| input | str | Yes | - | Input color node id |

**Returns:** Dict matching TextureProceduralNode with ToGrayscale op.

**Example:**
```python
grayscale_node("gray", "colored_input")
```

### palette_node()

Creates a palette quantization node.

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| id | str | Yes | - | Unique node identifier |
| input | str | Yes | - | Input node id |
| palette | list | Yes | - | List of hex colors to quantize to |

**Returns:** Dict matching TextureProceduralNode with Palette op.

**Example:**
```python
palette_node("retro", "colored", ["#000000", "#ff0000", "#00ff00", "#0000ff"])
```

### compose_rgba_node()

Creates an RGBA composition node.

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| id | str | Yes | - | Unique node identifier |
| r | str | Yes | - | Red channel node id |
| g | str | Yes | - | Green channel node id |
| b | str | Yes | - | Blue channel node id |
| a | str | No | None | Alpha channel node id |

**Returns:** Dict matching TextureProceduralNode with ComposeRgba op.

**Example:**
```python
compose_rgba_node("color", "red_channel", "green_channel", "blue_channel")
compose_rgba_node("color_with_alpha", "r", "g", "b", "alpha")
```

### normal_from_height_node()

Creates a normal map from height field node.

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| id | str | Yes | - | Unique node identifier |
| input | str | Yes | - | Input height field node id |
| strength | f64 | No | 1.0 | Normal map strength |

**Returns:** Dict matching TextureProceduralNode with NormalFromHeight op.

**Example:**
```python
normal_from_height_node("normals", "heightmap", 1.0)
```

### wang_tiles_node()

Creates a Wang tiles stochastic tiling node for seamless random tiling.

Wang tiles use edge-matching to create seamless random tiling from an input texture,
reducing visible repetition patterns.

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| id | str | Yes | - | Unique node identifier |
| input | str | Yes | - | Input texture node id |
| tile_count_x | int | No | 4 | Number of tiles in X direction |
| tile_count_y | int | No | 4 | Number of tiles in Y direction |
| blend_width | f64 | No | 0.1 | Blend width at edges (0.0-0.5) |

**Returns:** Dict matching TextureProceduralNode with WangTiles op.

**Example:**
```python
wang_tiles_node("tiled", "base_texture")
wang_tiles_node("tiled", "base_texture", 4, 4)
wang_tiles_node("tiled", "base_texture", 2, 2, 0.2)
```

### texture_bomb_node()

Creates a texture bombing (random splat) node.

Places randomized stamps of the input texture across the output,
with configurable density, scale variation, rotation, and blending.

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| id | str | Yes | - | Unique node identifier |
| input | str | Yes | - | Input texture node id to scatter |
| density | f64 | No | 0.5 | Stamp density (0.0-1.0) |
| scale_min | f64 | No | 0.8 | Minimum scale factor |
| scale_max | f64 | No | 1.2 | Maximum scale factor |
| rotation_variation | f64 | No | 0.0 | Rotation variation in degrees (0-360) |
| blend_mode | str | No | "max" | Blend mode: "max", "add", "average" |

**Returns:** Dict matching TextureProceduralNode with TextureBomb op.

**Example:**
```python
texture_bomb_node("scattered", "base_texture")
texture_bomb_node("scattered", "base_texture", 0.5)
texture_bomb_node("scattered", "base_texture", 0.7, 0.8, 1.2, 180.0, "add")
```

---

## Graph Functions

### texture_graph()

Creates a complete texture graph recipe params.

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| resolution | list | Yes | - | [width, height] in pixels |
| nodes | list | Yes | - | List of texture nodes |
| tileable | bool | No | True | Whether texture should tile |

**Returns:** Dict matching TextureProceduralV1Params.

**Example:**
```python
texture_graph(
    [64, 64],
    [
        noise_node("height", "perlin", 0.1, 4),
        threshold_node("mask", "height", 0.5)
    ]
)
```

---

## Trimsheet Functions

Trimsheet functions create texture atlas specs with deterministic shelf packing.

### trimsheet_tile()

Creates a tile definition for trimsheet packing.

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| id | str | Yes | - | Unique tile identifier |
| width | int | Yes | - | Tile width in pixels |
| height | int | Yes | - | Tile height in pixels |
| color | list | Yes | - | RGBA color [r, g, b, a], values 0.0-1.0 |

**Returns:** Dict matching TrimsheetTile.

**Example:**
```python
trimsheet_tile(id = "grass", width = 128, height = 128, color = [0.2, 0.6, 0.2, 1.0])
trimsheet_tile(id = "stone", width = 64, height = 64, color = [0.5, 0.5, 0.5, 1.0])
```

### trimsheet_spec()

Creates a complete trimsheet atlas spec with trimsheet_v1 recipe.

The trimsheet recipe uses deterministic shelf packing to arrange tiles into
a single atlas texture. It outputs both the PNG atlas and a JSON metadata file
containing UV coordinates for each tile.

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| asset_id | str | Yes | - | Kebab-case identifier for the asset |
| seed | int | Yes | - | Deterministic seed (0 to 2^32-1) |
| output_path | str | Yes | - | Output path for atlas PNG |
| resolution | list | Yes | - | [width, height] in pixels |
| tiles | list | Yes | - | List of tile definitions |
| metadata_path | str | No | None | Output path for UV metadata JSON |
| padding | int | No | 2 | Gutter pixels between tiles |
| description | str | No | None | Asset description |
| tags | list | No | None | Style tags |
| license | str | No | "CC0-1.0" | SPDX license identifier |

**Returns:** A complete spec dict ready for serialization.

**Features:**
- Deterministic shelf packing (tiles sorted by height, then width, then id)
- Mip-safe gutters (edge pixels replicated into padding)
- UV metadata output with normalized [0,1] coordinates

**Example:**
```python
trimsheet_spec(
    asset_id = "tileset-01",
    seed = 42,
    output_path = "atlas/tileset.png",
    metadata_path = "atlas/tileset.json",
    resolution = [1024, 1024],
    padding = 4,
    tiles = [
        trimsheet_tile(id = "grass", width = 128, height = 128, color = [0.2, 0.6, 0.2, 1.0]),
        trimsheet_tile(id = "stone", width = 64, height = 64, color = [0.5, 0.5, 0.5, 1.0]),
        trimsheet_tile(id = "water", width = 128, height = 64, color = [0.1, 0.3, 0.8, 1.0])
    ]
)
```

**Metadata Output Format:**
```json
{
  "atlas_width": 1024,
  "atlas_height": 1024,
  "padding": 4,
  "tiles": [
    {
      "id": "grass",
      "u_min": 0.00390625,
      "v_min": 0.00390625,
      "u_max": 0.12890625,
      "v_max": 0.12890625,
      "width": 128,
      "height": 128
    }
  ]
}
```

---

## Decal Functions

Decal functions create texture specs for decal/overlay textures with RGBA output,
optional normal and roughness maps, and placement metadata for game engine integration.

### decal_metadata()

Creates decal placement metadata for projection and rendering hints.

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| aspect_ratio | f64 | No | 1.0 | Width/height ratio for correct proportions |
| anchor | list | No | [0.5, 0.5] | Anchor point in normalized [0,1] coordinates |
| fade_distance | f64 | No | 0.0 | Edge fade distance (0.0-1.0), 0 = hard edges |
| projection_size | list | No | None | Optional world-space size [width, height] in meters |
| depth_range | list | No | None | Optional depth clipping [near, far] in meters |

**Returns:** Dict matching DecalMetadata.

**Example:**
```python
decal_metadata()
decal_metadata(aspect_ratio = 2.0, anchor = [0.5, 1.0], fade_distance = 0.1)
decal_metadata(projection_size = [1.0, 0.5], depth_range = [0.0, 0.1])
```

### decal_spec()

Creates a complete decal spec with decal_v1 recipe.

The decal recipe generates RGBA textures optimized for decal projection, with the alpha
channel composited from a separate mask. Optionally includes normal and roughness outputs
for PBR workflows, plus a JSON metadata sidecar for placement information.

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| asset_id | str | Yes | - | Kebab-case identifier for the asset |
| seed | int | Yes | - | Deterministic seed (0 to 2^32-1) |
| output_path | str | Yes | - | Output path for albedo PNG |
| resolution | list | Yes | - | [width, height] in pixels |
| nodes | list | Yes | - | List of texture nodes |
| albedo_output | str | Yes | - | Node id for albedo/diffuse output |
| alpha_output | str | Yes | - | Node id for alpha mask output |
| metadata | dict | Yes | - | Placement metadata from decal_metadata() |
| normal_output | str | No | None | Node id for normal map output |
| roughness_output | str | No | None | Node id for roughness output |
| normal_path | str | No | None | Output path for normal map PNG |
| roughness_path | str | No | None | Output path for roughness PNG |
| metadata_path | str | No | None | Output path for metadata JSON |
| description | str | No | None | Asset description |
| tags | list | No | None | Style tags |
| license | str | No | "CC0-1.0" | SPDX license identifier |

**Returns:** A complete spec dict ready for serialization.

**Features:**
- RGBA albedo output with alpha composited from separate mask node
- Optional normal map for surface detail
- Optional roughness map for PBR materials
- JSON metadata sidecar with placement hints (anchor, aspect ratio, fade, projection bounds)

**Example:**
```python
# Basic decal with just albedo and alpha
decal_spec(
    asset_id = "bullet-hole-01",
    seed = 42,
    output_path = "decals/bullet_hole.png",
    resolution = [512, 512],
    nodes = [
        noise_node("base", algorithm = "perlin", scale = 0.05),
        threshold_node("alpha", "base", threshold = 0.3)
    ],
    albedo_output = "base",
    alpha_output = "alpha",
    metadata = decal_metadata(aspect_ratio = 1.0, fade_distance = 0.1)
)

# Full PBR decal with normal, roughness, and metadata
decal_spec(
    asset_id = "blood-splatter-01",
    seed = 123,
    output_path = "decals/blood.png",
    normal_path = "decals/blood_normal.png",
    roughness_path = "decals/blood_roughness.png",
    metadata_path = "decals/blood.decal.json",
    resolution = [256, 256],
    nodes = [
        noise_node("base", algorithm = "simplex", scale = 0.1),
        threshold_node("alpha", "base", threshold = 0.4),
        normal_from_height_node("normal", "base", strength = 0.5),
        constant_node("rough", 0.8)
    ],
    albedo_output = "base",
    alpha_output = "alpha",
    normal_output = "normal",
    roughness_output = "rough",
    metadata = decal_metadata(
        aspect_ratio = 1.5,
        anchor = [0.5, 0.5],
        fade_distance = 0.2,
        projection_size = [0.5, 0.33]
    )
)
```

**Metadata Output Format:**
```json
{
  "resolution": [256, 256],
  "aspect_ratio": 1.5,
  "anchor": [0.5, 0.5],
  "fade_distance": 0.2,
  "projection_size": [0.5, 0.33],
  "has_normal_map": true,
  "has_roughness_map": true
}
```

---

## Splat Set Functions

Splat set functions create terrain texture sets with multiple material layers,
blend masks (splat maps), per-layer PBR outputs, and macro variation overlays.

### splat_layer()

Creates a terrain splat layer definition.

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| id | str | Yes | - | Unique layer identifier (e.g., "grass", "dirt", "rock") |
| albedo_color | list | Yes | - | Base color [r, g, b, a] (0.0-1.0) |
| normal_strength | f64 | No | 1.0 | Normal map strength |
| roughness | f64 | No | 0.8 | Roughness value (0.0-1.0) |
| detail_scale | f64 | No | 0.2 | Detail noise scale |
| detail_intensity | f64 | No | 0.3 | Detail noise intensity |

**Returns:** Dict matching SplatLayer.

**Example:**
```python
splat_layer(id = "grass", albedo_color = [0.2, 0.5, 0.1, 1.0])
splat_layer(id = "dirt", albedo_color = [0.4, 0.3, 0.2, 1.0], roughness = 0.9)
splat_layer(id = "rock", albedo_color = [0.5, 0.5, 0.5, 1.0], roughness = 0.7, normal_strength = 0.8)
```

### splat_set_spec()

Creates a complete splat set spec with splat_set_v1 recipe.

The splat set recipe generates terrain texture sets including:
- Per-layer albedo, normal, and roughness textures
- RGBA splat mask textures (up to 4 layers per mask)
- Optional macro variation texture for large-scale detail
- JSON metadata with layer info and mask channel assignments

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| asset_id | str | Yes | - | Kebab-case identifier for the asset |
| seed | int | Yes | - | Deterministic seed (0 to 2^32-1) |
| resolution | list | Yes | - | [width, height] in pixels |
| layers | list | Yes | - | List of splat_layer() definitions (max 4 per mask) |
| output_prefix | str | Yes | - | Output path prefix for generated textures |
| mask_mode | str | No | "noise" | "noise", "height", "slope", "height_slope" |
| noise_scale | f64 | No | 0.1 | Noise scale for noise-based blending |
| macro_variation | bool | No | False | Generate macro variation texture |
| macro_scale | f64 | No | 0.05 | Macro variation scale |
| macro_intensity | f64 | No | 0.3 | Macro variation intensity (0.0-1.0) |
| metadata_path | str | No | None | Output path for metadata JSON |
| description | str | No | None | Asset description |
| tags | list | No | None | Style tags |
| license | str | No | "CC0-1.0" | SPDX license identifier |

**Returns:** A complete spec dict ready for serialization.

**Mask Modes:**
- `noise`: Pure noise-based blending (uniform distribution)
- `height`: Height-based blending (lower layers at bottom, higher at top)
- `slope`: Slope-based blending (flat areas vs steep areas)
- `height_slope`: Combined height and slope blending

**Example:**
```python
# Basic terrain with grass and dirt
splat_set_spec(
    asset_id = "terrain-basic-01",
    seed = 42,
    resolution = [512, 512],
    layers = [
        splat_layer(id = "grass", albedo_color = [0.2, 0.5, 0.1, 1.0], roughness = 0.8),
        splat_layer(id = "dirt", albedo_color = [0.4, 0.3, 0.2, 1.0], roughness = 0.9)
    ],
    output_prefix = "terrain/basic"
)

# Full terrain with 4 layers, macro variation, and metadata
splat_set_spec(
    asset_id = "terrain-full-01",
    seed = 123,
    resolution = [1024, 1024],
    layers = [
        splat_layer(id = "grass", albedo_color = [0.2, 0.5, 0.1, 1.0], roughness = 0.8),
        splat_layer(id = "dirt", albedo_color = [0.4, 0.3, 0.2, 1.0], roughness = 0.9),
        splat_layer(id = "rock", albedo_color = [0.5, 0.5, 0.5, 1.0], roughness = 0.7),
        splat_layer(id = "sand", albedo_color = [0.8, 0.7, 0.5, 1.0], roughness = 0.6)
    ],
    output_prefix = "terrain/full",
    mask_mode = "height_slope",
    macro_variation = True,
    macro_intensity = 0.4,
    metadata_path = "terrain/full.splat.json"
)
```

**Generated Outputs:**
```
terrain/basic_grass.albedo.png
terrain/basic_grass.normal.png
terrain/basic_grass.roughness.png
terrain/basic_dirt.albedo.png
terrain/basic_dirt.normal.png
terrain/basic_dirt.roughness.png
terrain/basic_mask0.png          # RGBA: R=grass, G=dirt
```

**Metadata Output Format:**
```json
{
  "resolution": [512, 512],
  "layers": [
    {
      "id": "grass",
      "mask_index": 0,
      "mask_channel": 0,
      "roughness": 0.8
    },
    {
      "id": "dirt",
      "mask_index": 0,
      "mask_channel": 1,
      "roughness": 0.9
    }
  ],
  "mask_mode": "noise",
  "has_macro_variation": false,
  "splat_mask_count": 1
}
```

---

## Matcap Functions

### matcap_v1()

Creates a complete matcap texture spec with matcap_v1 recipe.

Matcaps (material capture) are 2D textures that encode lighting and shading for NPR (non-photorealistic rendering). They map surface normals to colors, providing fast stylized rendering.

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| asset_id | str | Yes | - | Kebab-case asset identifier |
| seed | int | Yes | - | Deterministic seed (0 to 2^32-1) |
| output_path | str | Yes | - | Output file path for PNG |
| resolution | list | Yes | - | [width, height] in pixels (typically square) |
| preset | str | Yes | - | Matcap preset (see below) |
| base_color | list | No | None | RGB color override [r, g, b] (0.0-1.0) |
| toon_steps | int | No | None | Toon shading steps (2-16) |
| outline_width | int | No | None | Outline width in pixels (1-10) |
| outline_color | list | No | None | Outline color [r, g, b] (0.0-1.0) |
| curvature_enabled | bool | No | False | Enable curvature masking |
| curvature_strength | f64 | No | 0.5 | Curvature mask strength (0.0-1.0) |
| cavity_enabled | bool | No | False | Enable cavity masking |
| cavity_strength | f64 | No | 0.5 | Cavity mask strength (0.0-1.0) |
| description | str | No | None | Asset description |
| tags | list | No | None | Style tags |
| license | str | No | "CC0-1.0" | SPDX license identifier |

**Valid Presets:**
- `"toon_basic"` - Basic toon shading with clear light/shadow separation
- `"toon_rim"` - Toon shading with rim lighting highlight
- `"metallic"` - Metallic shading with strong specular highlights
- `"ceramic"` - Ceramic/porcelain shading with soft diffuse falloff
- `"clay"` - Matte clay shading with no specular
- `"skin"` - Skin/subsurface shading with soft transitions
- `"plastic"` - Glossy plastic with sharp highlights
- `"velvet"` - Velvet/fabric with anisotropic-like highlights

**Returns:** Complete spec dict matching SpecCade spec schema.

**Example:**
```python
# Basic toon matcap
matcap_v1(
    asset_id = "toon-red-01",
    seed = 42,
    output_path = "matcaps/toon_red.png",
    resolution = [512, 512],
    preset = "toon_basic",
    base_color = [0.8, 0.2, 0.2],
    toon_steps = 4
)

# Metallic matcap with outline
matcap_v1(
    asset_id = "metal-chrome",
    seed = 123,
    output_path = "matcaps/chrome.png",
    resolution = [256, 256],
    preset = "metallic",
    outline_width = 2,
    outline_color = [0.0, 0.0, 0.0]
)

# Clay matcap with curvature and cavity
matcap_v1(
    asset_id = "clay-stylized",
    seed = 456,
    output_path = "matcaps/clay.png",
    resolution = [512, 512],
    preset = "clay",
    base_color = [0.8, 0.6, 0.5],
    curvature_enabled = True,
    curvature_strength = 0.6,
    cavity_enabled = True,
    cavity_strength = 0.4,
    description = "Stylized clay matcap",
    tags = ["clay", "stylized", "npr"]
)
```

**Generated Spec Structure:**
```json
{
  "spec_version": 1,
  "asset_id": "toon-red-01",
  "asset_type": "texture",
  "license": "CC0-1.0",
  "seed": 42,
  "outputs": [
    {
      "kind": "primary",
      "format": "png",
      "path": "matcaps/toon_red.png"
    }
  ],
  "recipe": {
    "kind": "texture.matcap_v1",
    "params": {
      "resolution": [512, 512],
      "preset": "toon_basic",
      "base_color": [0.8, 0.2, 0.2],
      "toon_steps": 4
    }
  }
}
```

---

## Material Preset Functions

### material_preset_v1()

Creates a complete material preset spec with texture.material_preset_v1 recipe.

Material presets generate multiple PBR texture outputs (albedo, roughness, metallic, normal) from predefined style presets with optional parameter overrides. This provides a "preset + parameterization" approach for consistent art direction.

**Parameters:**
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| asset_id | str | Yes | - | Kebab-case asset identifier |
| seed | int | Yes | - | Deterministic seed (0 to 2^32-1) |
| output_prefix | str | Yes | - | Output path prefix for generated textures |
| resolution | list | Yes | - | [width, height] in pixels |
| preset | str | Yes | - | Material preset (see below) |
| tileable | bool | No | True | Whether textures tile seamlessly |
| base_color | list | No | None | RGB color override [r, g, b] (0.0-1.0) |
| roughness_range | list | No | None | Roughness range [min, max] (0.0-1.0) |
| metallic | f64 | No | None | Metallic value (0.0-1.0) |
| noise_scale | f64 | No | None | Noise scale for detail patterns |
| pattern_scale | f64 | No | None | Pattern scale for macro features |
| description | str | No | None | Asset description |
| tags | list | No | None | Style tags |
| license | str | No | "CC0-1.0" | SPDX license identifier |

**Valid Presets:**
- `"toon_metal"` - Flat albedo with rim highlights, stepped roughness for stylized metal
- `"stylized_wood"` - Wood grain pattern with warm tones, organic noise
- `"neon_glow"` - Dark base with bright emissive-style highlights
- `"ceramic_glaze"` - Smooth, high-gloss ceramic/porcelain look
- `"sci_fi_panel"` - Geometric patterns with metallic panels and panel lines
- `"clean_plastic"` - Uniform albedo with medium roughness for clean plastic
- `"rough_stone"` - Rocky noise patterns with high roughness for stone surfaces
- `"brushed_metal"` - Directional anisotropic streaks for brushed metal

**Returns:** Complete spec dict with multiple outputs (albedo, roughness, metallic, normal, metadata).

**Example:**
```python
# Basic toon metal preset
material_preset_v1(
    asset_id = "metal-panel-01",
    seed = 42,
    output_prefix = "materials/metal_panel",
    resolution = [512, 512],
    preset = "toon_metal"
)

# Stylized wood with custom color and roughness
material_preset_v1(
    asset_id = "wood-plank-01",
    seed = 123,
    output_prefix = "materials/wood_plank",
    resolution = [1024, 1024],
    preset = "stylized_wood",
    tileable = True,
    base_color = [0.7, 0.5, 0.3],
    roughness_range = [0.5, 0.9],
    description = "Stylized wood plank material",
    tags = ["wood", "stylized", "pbr"]
)

# Sci-fi panel with metallic override
material_preset_v1(
    asset_id = "scifi-wall-01",
    seed = 456,
    output_prefix = "materials/scifi_wall",
    resolution = [512, 512],
    preset = "sci_fi_panel",
    metallic = 0.9,
    noise_scale = 0.05,
    pattern_scale = 0.3
)
```

**Generated Outputs:**
```
materials/metal_panel_albedo.png      # RGB albedo texture
materials/metal_panel_roughness.png   # Grayscale roughness map
materials/metal_panel_metallic.png    # Grayscale metallic map
materials/metal_panel_normal.png      # RGB normal map
materials/metal_panel.material.json   # Metadata sidecar
```

**Metadata Output Format:**
```json
{
  "resolution": [512, 512],
  "tileable": true,
  "preset": "toon_metal",
  "base_color": [0.85, 0.85, 0.9],
  "roughness_range": [0.2, 0.5],
  "metallic": 0.9,
  "generated_maps": ["albedo", "roughness", "metallic", "normal"]
}
```

---

[← Back to Index](stdlib-reference.md)
