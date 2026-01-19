# SpecCade Starlark Standard Library - Texture Functions

[← Back to Index](stdlib-reference.md)

## Texture Functions

Texture functions provide a node-based procedural texture graph system.

## Table of Contents
- [Node Functions](#node-functions)
- [Graph Functions](#graph-functions)
- [Trimsheet Functions](#trimsheet-functions)

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

[← Back to Index](stdlib-reference.md)
