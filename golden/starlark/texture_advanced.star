# Texture advanced coverage example
#
# Demonstrates texture stdlib functions for advanced texture operations.
# Covers: texture_spec, grayscale_node, palette_node, compose_rgba_node,
#         decal_metadata, decal_spec, splat_layer, splat_set_spec,
#         trimsheet_spec, trimsheet_tile

# grayscale_node converts color to grayscale
gray = grayscale_node(
    id = "gray",
    input = "noise1",
    method = "luminance"
)

# palette_node quantizes to a color palette
pal = palette_node(
    id = "palettized",
    input = "noise1",
    colors = ["#1a1a2e", "#16213e", "#0f3460", "#e94560"]
)

# compose_rgba_node combines channels into RGBA
rgba = compose_rgba_node(
    id = "composed",
    r = "noise1",
    g = "noise2",
    b = "gray",
    a = "const_alpha"
)

# decal_metadata for placement info
decal_meta = decal_metadata(
    aspect_ratio = 2.0,
    projection = "planar",
    blend_mode = "multiply"
)

# decal_spec creates a complete decal texture
decal_spec(
    asset_id = "stdlib-decal-coverage-01",
    albedo_nodes = [
        noise_node("base", "perlin", 4.0),
        color_ramp_node("colored", "base", ["#ff0000", "#00ff00"])
    ],
    normal_strength = 0.5,
    output_prefix = "textures/decal_coverage",
    metadata = decal_metadata(1.5, "cylindrical", "overlay")
)

# splat_layer defines a terrain splat layer
layer1 = splat_layer(
    id = "grass",
    albedo = "#4a7c23",
    roughness = 0.7,
    normal_strength = 0.3
)

layer2 = splat_layer(
    id = "dirt",
    albedo = "#8b6914",
    roughness = 0.9,
    normal_strength = 0.5
)

# splat_set_spec creates a terrain splat set
splat_set_spec(
    asset_id = "stdlib-splat-set-coverage-01",
    layers = [layer1, layer2],
    resolution = 512,
    output_prefix = "textures/splat_coverage"
)

# trimsheet_tile creates a trimsheet tile definition
tile1 = trimsheet_tile(
    id = "metal_panel",
    color = "#888888",
    roughness = 0.3,
    metallic = 0.9
)

tile2 = trimsheet_tile(
    id = "wood_plank",
    color = "#8b6914",
    roughness = 0.7,
    metallic = 0.0
)

# trimsheet_spec creates a complete trimsheet
trimsheet_spec(
    asset_id = "stdlib-trimsheet-coverage-01",
    tiles = [tile1, tile2],
    resolution = 1024,
    tile_size = 128,
    output_prefix = "textures/trimsheet_coverage"
)

# texture_spec creates a complete procedural texture spec
texture_spec(
    asset_id = "stdlib-texture-spec-coverage-01",
    resolution = [256, 256],
    nodes = [
        noise_node("noise1", "perlin", 4.0),
        noise_node("noise2", "simplex", 8.0),
        grayscale_node("gray", "noise1", "luminance"),
        palette_node("palettized", "gray", ["#000000", "#444444", "#888888", "#ffffff"]),
        const_node("const_alpha", 1.0),
        compose_rgba_node("composed", "noise1", "noise2", "gray", "const_alpha"),
        color_ramp_node("colored", "palettized", ["#1a1a2e", "#e94560"])
    ],
    output_node = "colored",
    output_path = "textures/advanced_coverage.png"
)
