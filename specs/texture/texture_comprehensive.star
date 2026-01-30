# Comprehensive Tier-1 procedural texture fixture
# Exercises multiple texture graph nodes while staying deterministic.

spec(
    asset_id = "golden-texture-comprehensive-01",
    asset_type = "texture",
    seed = 4242,
    description = "Comprehensive procedural texture fixture (Tier 1)",
    outputs = [output("textures/texture_comprehensive.png", "png", source = "pal")],
    recipe = {
        "kind": "texture.procedural_v1",
        "params": texture_graph(
            [256, 256],
            [
                noise_node("base_noise", "perlin", 0.08, 4),
                wang_tiles_node("tiled", "base_noise", 4, 4, 0.1),

                noise_node("scatter_src", "worley", 0.15, 2),
                texture_bomb_node("scattered", "scatter_src", 0.5, 0.75, 1.25, 45.0, "max"),
                multiply_node("detail", "tiled", "scattered"),
                clamp_node("r", "detail", 0.0, 1.0),

                gradient_node("grad", "radial", 0.0, 1.0),
                stripes_node("stripes", "horizontal", 10, 0.2, 0.9),
                multiply_node("g", "grad", "stripes"),

                checkerboard_node("checker", 12, 0.0, 1.0),
                invert_node("inv_checker", "checker"),
                lerp_node("b", "checker", "inv_checker", "tiled"),

                threshold_node("a", "detail", 0.55),

                compose_rgba_node("rgba", "r", "g", "b", "a"),
                grayscale_node("gray", "rgba"),
                color_ramp_node(
                    "colored",
                    "gray",
                    ["#0b1d2a", "#1f5a63", "#f2f0d8"],
                ),
                palette_node(
                    "pal",
                    "colored",
                    ["#0b1d2a", "#1f5a63", "#6bb39a", "#f2f0d8"],
                ),
            ],
            True,
        ),
    },
)
