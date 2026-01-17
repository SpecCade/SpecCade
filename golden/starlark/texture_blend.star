# Texture blending operations example
#
# Demonstrates various ways to combine texture nodes.
# Add combines values, multiply masks one by another, lerp interpolates.
# Covers: add_node(), multiply_node(), lerp_node()

spec(
    asset_id = "stdlib-texture-blend-01",
    asset_type = "texture",
    seed = 42,
    outputs = [output("textures/blend.png", "png", source = "colored")],
    recipe = {
        "kind": "texture.procedural_v1",
        "params": texture_graph(
            [128, 128],
            [
                # Two noise sources
                noise_node("noise1", "perlin", 0.05, 4, 0.5, 2.0),
                noise_node("noise2", "simplex", 0.1, 3, 0.6, 2.0),

                # Gradient for masking
                gradient_node("mask", "horizontal", 0.0, 1.0),

                # Add the two noises together
                add_node("added", "noise1", "noise2"),

                # Multiply to apply vignette-like mask
                multiply_node("masked", "added", "mask"),

                # Lerp between noise patterns using gradient
                lerp_node("blended", "noise1", "noise2", "mask"),

                # Final color ramp
                color_ramp_node("colored", "blended", ["#1a1a2e", "#16213e", "#0f3460", "#e94560"])
            ],
            True
        )
    }
)
