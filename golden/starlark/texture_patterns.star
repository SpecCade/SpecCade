# Texture patterns example
#
# Demonstrates geometric pattern generators for textures.
# Stripes create lines, checkerboard creates alternating tiles.
# Covers: stripes_node(), checkerboard_node()

spec(
    asset_id = "stdlib-texture-patterns-01",
    asset_type = "texture",
    seed = 42,
    outputs = [output("textures/patterns.png", "png", source = "colored")],
    recipe = {
        "kind": "texture.procedural_v1",
        "params": texture_graph(
            [128, 128],
            [
                # Horizontal stripes
                stripes_node("hstripes", "horizontal", 8, 0.2, 0.8),

                # Vertical stripes
                stripes_node("vstripes", "vertical", 16, 0.0, 1.0),

                # Checkerboard pattern
                checkerboard_node("checker", 16, 0.0, 1.0),

                # Combine stripes using multiply
                multiply_node("grid", "hstripes", "vstripes"),

                # Blend with checkerboard
                add_node("combined", "grid", "checker"),

                # Color the result
                color_ramp_node("colored", "combined", ["#000000", "#ffffff"])
            ],
            True
        )
    }
)
