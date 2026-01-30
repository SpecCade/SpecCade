# Golden test: Misc texture nodes
# Tests: invert_node, threshold_node, add_node, multiply_node, clamp_node

spec(
    asset_id = "stdlib-texture-nodes-misc-01",
    asset_type = "texture",
    seed = 42,
    outputs = [output("textures/nodes_misc.png", "png", source = "clamped")],
    recipe = {
        "kind": "texture.procedural_v1",
        "params": texture_graph(
            [256, 256],
            [
                noise_node("noise1", "perlin", 4.0),
                noise_node("noise2", "simplex", 8.0),
                invert_node("inverted", "noise1"),
                threshold_node("thresholded", "noise2", 0.5),
                add_node("added", "noise1", "noise2"),
                multiply_node("multiplied", "noise1", "thresholded"),
                clamp_node("clamped", "added", 0.2, 0.8)
            ],
            True
        )
    }
)
