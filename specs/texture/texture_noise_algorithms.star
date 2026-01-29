# Noise algorithm coverage - demonstrates all noise algorithm variants
#
# This example covers the 'value' and 'fbm' algorithm enum values for noise_node.
# Combined with other specs using 'perlin', 'simplex', and 'worley', this achieves
# full enum coverage for the algorithm parameter.

# Value noise texture
spec(
    asset_id = "stdlib-texture-value-noise-01",
    asset_type = "texture",
    seed = 42,
    outputs = [output("textures/value_noise.png", "png", source = "noise")],
    recipe = {
        "kind": "texture.procedural_v1",
        "params": texture_graph(
            [64, 64],
            [noise_node("noise", "value", 0.1, 4, 0.5, 2.0)],
            True
        )
    },
    description = "Value noise texture - enum coverage for algorithm::value"
)

# FBM (Fractal Brownian Motion) noise texture
spec(
    asset_id = "stdlib-texture-fbm-noise-01",
    asset_type = "texture",
    seed = 43,
    outputs = [output("textures/fbm_noise.png", "png", source = "noise")],
    recipe = {
        "kind": "texture.procedural_v1",
        "params": texture_graph(
            [64, 64],
            [noise_node("noise", "fbm", 0.1, 4, 0.5, 2.0)],
            True
        )
    },
    description = "FBM noise texture - enum coverage for algorithm::fbm"
)
