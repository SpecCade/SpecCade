# Adversarial: Texture with [0, 0] resolution
# Expected: validation rejects zero-sized textures

spec(
    asset_id = "adv-zero-res-texture",
    asset_type = "texture",
    license = "CC0-1.0",
    seed = 99904,
    outputs = [output("textures/zero.png", "png")],
    recipe = {
        "kind": "texture.procedural_v1",
        "params": texture_graph(
            [0, 0],
            [noise_node("out", "simplex", 0.1, 4)]
        )
    }
)
