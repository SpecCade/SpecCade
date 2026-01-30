# Comprehensive Tier-1 normal map fixture
# Builds a height field using multiple nodes, then converts to normals.

spec(
    asset_id = "golden-normal-comprehensive-01",
    asset_type = "texture",
    seed = 4243,
    description = "Comprehensive normal-map fixture derived from height field (Tier 1)",
    outputs = [output("textures/normal_comprehensive.png", "png", source = "normals")],
    recipe = {
        "kind": "texture.procedural_v1",
        "params": texture_graph(
            [256, 256],
            [
                noise_node("base", "perlin", 0.06, 5, 0.5, 2.0),
                noise_node("detail", "simplex", 0.15, 3, 0.6, 2.2),
                add_node("sum", "base", "detail"),

                gradient_node("mask", "radial", 0.0, 1.0),
                multiply_node("masked", "sum", "mask"),

                noise_node("rocks", "worley", 0.25, 2),
                texture_bomb_node("bombed", "rocks", 0.35, 0.6, 1.4, 30.0, "max"),

                add_node("height_raw", "masked", "bombed"),
                clamp_node("height", "height_raw", 0.0, 1.0),
                normal_from_height_node("normals", "height", 2.0),
            ],
            True,
        ),
    },
)
