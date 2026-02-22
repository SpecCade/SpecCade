# SpecCade Starlark Standard Library - Texture Functions

[← Back to Index](stdlib-reference.md)

> **SSOT:** For complete parameter details, use `speccade stdlib dump --format json`
> or see the Rust types in `crates/speccade-spec/src/recipe/texture/`.

Textures use a node-based procedural graph system. Nodes are connected by ID references.

## Node Functions

| Function | Description |
|----------|-------------|
| `noise_node(id, algorithm, scale, octaves, persistence, lacunarity)` | Noise generator (perlin, simplex, worley, value, gabor, fbm) |
| `reaction_diffusion_preset(preset)` | Tuned Gray-Scott parameter presets (mitosis, worms, spots) |
| `reaction_diffusion_node(id, steps, feed, kill, diffuse_a, diffuse_b, dt, seed_density)` | Gray-Scott reaction-diffusion pattern |
| `gradient_node(id, direction, start, end, center, inner, outer)` | Gradient (horizontal, vertical, radial) |
| `constant_node(id, value)` | Constant value |
| `threshold_node(id, input, threshold)` | Binary threshold |
| `invert_node(id, input)` | Invert (1 - x) |
| `color_ramp_node(id, input, ramp)` | Map values to color gradient |
| `add_node(id, a, b)` | Add blend (a + b) |
| `multiply_node(id, a, b)` | Multiply blend (a * b) |
| `lerp_node(id, a, b, t)` | Linear interpolation |
| `clamp_node(id, input, min, max)` | Clamp to range |
| `stripes_node(id, direction, stripe_width, color1, color2)` | Stripe pattern |
| `checkerboard_node(id, tile_size, color1, color2)` | Checkerboard pattern |
| `grayscale_node(id, input)` | Convert to grayscale |
| `palette_node(id, input, palette)` | Palette quantization |
| `compose_rgba_node(id, r, g, b, a)` | RGBA channel composition |
| `normal_from_height_node(id, input, strength)` | Normal map from heightfield |
| `wang_tiles_node(id, input, tile_count_x, tile_count_y, blend_width)` | Stochastic tiling |
| `texture_bomb_node(id, input, density, scale_min, scale_max, ...)` | Random scatter/splat |

## Graph Functions

| Function | Description |
|----------|-------------|
| `texture_graph(resolution, nodes, tileable)` | Assemble a procedural texture graph |

**Example:**
```python
texture_graph([64, 64], [
    noise_node("h", "perlin", 0.1, 4),
    threshold_node("m", "h", 0.5),
])
```

## Specialized Recipes

| Function | Description |
|----------|-------------|
| `trimsheet_spec(...)` | Texture atlas with shelf packing and UV metadata |
| `trimsheet_tile(id, width, height, color)` | Tile definition for trimsheet |
| `decal_spec(...)` | Decal texture with RGBA, optional normal/roughness, placement metadata |
| `decal_metadata(...)` | Decal placement hints |
| `splat_set_spec(...)` | Terrain splat set (per-layer PBR + blend masks) |
| `splat_layer(id, albedo_color, ...)` | Terrain layer definition |
| `matcap_v1(...)` | Matcap (material capture) texture for NPR rendering |
| `material_preset_v1(...)` | PBR material from preset (toon_metal, stylized_wood, etc.) |

Presets for `matcap_v1`: toon_basic, toon_rim, metallic, ceramic, clay, skin, plastic, velvet.

Presets for `material_preset_v1`: toon_metal, stylized_wood, neon_glow, ceramic_glaze, sci_fi_panel, clean_plastic, rough_stone, brushed_metal.

[← Back to Index](stdlib-reference.md)
