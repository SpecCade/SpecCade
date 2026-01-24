# Matcap texture generation test
#
# Tests the texture.matcap_v1 recipe with various presets and options.

matcap_v1(
    asset_id = "matcap-toon-basic",
    seed = 12345,
    output_path = "matcaps/toon_basic.png",
    resolution = [256, 256],
    preset = "toon_basic",
    base_color = [0.8, 0.2, 0.2],
    toon_steps = 4,
    outline_width = 2,
    outline_color = [0.0, 0.0, 0.0],
    description = "Basic toon matcap with red base color and outline",
    tags = ["toon", "stylized", "red"],
    license = "CC0-1.0"
)
