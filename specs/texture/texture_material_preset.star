# Material preset texture generation test
#
# Tests the texture.material_preset_v1 recipe with ToonMetal preset
# and custom color override.

material_preset_v1(
    asset_id = "material-toon-metal",
    seed = 12345,
    output_prefix = "materials/toon_metal",
    resolution = [256, 256],
    preset = "toon_metal",
    base_color = [0.9, 0.85, 0.8],
    description = "Toon metal material with warm base color override",
    style_tags = ["toon", "metal", "stylized"],
    license = "CC0-1.0"
)
