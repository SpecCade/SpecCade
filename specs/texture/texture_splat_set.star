# Splat set golden spec
#
# Exercises the texture.splat_set_v1 recipe with 4 terrain layers (RGBA splat
# mask channels), noise-based mask generation, and macro variation overlay.

grass = splat_layer(
    id = "grass",
    albedo_color = [0.22, 0.52, 0.12, 1.0],
    normal_strength = 0.8,
    roughness = 0.75,
    detail_scale = 4.0,
    detail_intensity = 0.35
)

dirt = splat_layer(
    id = "dirt",
    albedo_color = [0.45, 0.35, 0.22, 1.0],
    normal_strength = 0.65,
    roughness = 0.90,
    detail_scale = 6.0,
    detail_intensity = 0.40
)

rock = splat_layer(
    id = "rock",
    albedo_color = [0.50, 0.48, 0.45, 1.0],
    normal_strength = 1.0,
    roughness = 0.55,
    detail_scale = 2.0,
    detail_intensity = 0.20
)

sand = splat_layer(
    id = "sand",
    albedo_color = [0.82, 0.72, 0.50, 1.0],
    normal_strength = 0.4,
    roughness = 0.85,
    detail_scale = 8.0,
    detail_intensity = 0.25
)

splat_set_spec(
    asset_id = "golden-texture-splat-set-01",
    seed = 3003,
    resolution = [512, 512],
    layers = [grass, dirt, rock, sand],
    output_prefix = "terrain/highland_splat",
    mask_mode = "noise",
    noise_scale = 6.0,
    macro_variation = True,
    macro_scale = 24.0,
    macro_intensity = 0.35,
    description = "Highland terrain splat set: 4 layers (grass/dirt/rock/sand), noise-blended with macro variation",
    tags = ["golden", "splat", "terrain"]
)
