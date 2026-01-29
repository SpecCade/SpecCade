# Colored noise texture with gradient - demonstrates color_ramp_node
#
# This example creates a more complex texture:
# 1. Generate simplex noise
# 2. Multiply with a radial gradient for vignette effect
# 3. Map through a color ramp

spec(
    asset_id = "stdlib-texture-colored-01",
    asset_type = "texture",
    seed = 42,
    outputs = [output("textures/colored.png", "png", source = "colored")],
    recipe = {
        "kind": "texture.procedural_v1",
        "params": texture_graph(
            [128, 128],
            [
                noise_node("n", "simplex", 0.08, 6),
                gradient_node("g", "radial", 1.0, 0.2),
                color_ramp_node("colored", "n", ["#1a1a2e", "#16213e", "#0f3460", "#e94560"])
            ]
        )
    }
)
