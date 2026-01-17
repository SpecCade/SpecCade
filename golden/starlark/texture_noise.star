# Procedural noise texture - demonstrates texture graph stdlib
#
# This example creates a tileable noise texture using the procedural graph system.
# The noise_node() creates Perlin noise, and threshold_node() converts it to a binary mask.

spec(
    asset_id = "stdlib-texture-noise-01",
    asset_type = "texture",
    seed = 42,
    outputs = [output("textures/noise.png", "png", source = "mask")],
    recipe = {
        "kind": "texture.procedural_v1",
        "params": texture_graph(
            [64, 64],
            [
                noise_node("height", "perlin", 0.1, 4, 0.5, 2.0),
                threshold_node("mask", "height", 0.5)
            ],
            True
        )
    }
)
