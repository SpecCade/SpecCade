# Texture advanced coverage example
#
# Demonstrates texture stdlib functions for advanced texture operations.
# Covers: texture_spec, grayscale_node, palette_node, compose_rgba_node, decal_metadata,
#         decal_spec, splat_layer, splat_set_spec, trimsheet_spec, trimsheet_tile

# grayscale_node converts color to grayscale
gray = grayscale_node("gray", "noise1")

# palette_node quantizes to a color palette
pal = palette_node("palettized", "noise1", ["#1a1a2e", "#16213e", "#0f3460", "#e94560"])

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
    aspect_ratio = 2.0
)

# texture_spec creates a complete procedural texture spec
texture_spec(
    asset_id = "stdlib-texture-spec-coverage-01",
    seed = 42,
    resolution = [256, 256],
    nodes = [
        noise_node("noise1", "perlin", 4.0),
        noise_node("noise2", "simplex", 8.0),
        grayscale_node("gray", "noise1"),
        palette_node("palettized", "gray", ["#000000", "#444444", "#888888", "#ffffff"]),
        constant_node("const_alpha", 1.0),
        compose_rgba_node("composed", "noise1", "noise2", "gray", "const_alpha"),
        color_ramp_node("colored", "palettized", ["#1a1a2e", "#e94560"])
    ],
    output_path = "textures/advanced_coverage.png",
    format = "png"
)

# --------------------------------------------------------------------------
# Decal spec coverage
# --------------------------------------------------------------------------

# decal_spec creates a complete decal with decal_v1 recipe
decal_spec(
    asset_id = "stdlib-decal-spec-coverage-01",
    seed = 42,
    output_path = "textures/decal_blood_splatter.png",
    resolution = [256, 256],
    nodes = [
        noise_node("base", "worley", 8.0),
        threshold_node("thresh", "base", 0.5)
    ],
    albedo_output = "thresh",
    alpha_output = "thresh",
    metadata = decal_metadata(aspect_ratio = 1.0)
)

# --------------------------------------------------------------------------
# Splat set coverage
# --------------------------------------------------------------------------

# splat_layer creates a terrain splat layer definition
grass_layer = splat_layer(
    id = "grass",
    albedo_color = [0.2, 0.5, 0.15, 1.0],
    normal_strength = 0.8,
    roughness = 0.7,
    detail_scale = 4.0,
    detail_intensity = 0.3
)

dirt_layer = splat_layer(
    id = "dirt",
    albedo_color = [0.4, 0.3, 0.2, 1.0],
    normal_strength = 0.6,
    roughness = 0.9
)

rock_layer = splat_layer(
    id = "rock",
    albedo_color = [0.5, 0.5, 0.5, 1.0],
    normal_strength = 1.0,
    roughness = 0.5
)

# splat_set_spec creates a complete splat set spec with splat_set_v1 recipe
splat_set_spec(
    asset_id = "stdlib-splat-set-coverage-01",
    seed = 42,
    resolution = [512, 512],
    layers = [grass_layer, dirt_layer, rock_layer],
    output_prefix = "terrain/meadow_splat",
    mask_mode = "noise",
    noise_scale = 8.0,
    macro_variation = True,
    macro_scale = 32.0
)

# --------------------------------------------------------------------------
# Trimsheet coverage
# --------------------------------------------------------------------------

# trimsheet_tile creates a trimsheet tile definition with a solid color
tile_wood = trimsheet_tile(
    id = "wood_planks",
    width = 128,
    height = 64,
    color = [0.6, 0.4, 0.25, 1.0]
)

tile_metal = trimsheet_tile(
    id = "metal_panel",
    width = 128,
    height = 64,
    color = [0.4, 0.4, 0.45, 1.0]
)

tile_concrete = trimsheet_tile(
    id = "concrete",
    width = 128,
    height = 128,
    color = [0.5, 0.5, 0.5, 1.0]
)

# trimsheet_spec creates a complete trimsheet spec with trimsheet_v1 recipe
trimsheet_spec(
    asset_id = "stdlib-trimsheet-coverage-01",
    seed = 42,
    output_path = "textures/trimsheet_industrial.png",
    resolution = [512, 256],
    tiles = [tile_wood, tile_metal, tile_concrete],
    padding = 2,
    description = "Industrial modular trim sheet",
    tags = ["industrial", "modular"]
)
