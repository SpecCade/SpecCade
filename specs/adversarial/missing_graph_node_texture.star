# Adversarial: Texture referencing nonexistent node ID in output source
# Expected: validation rejects references to undefined nodes

spec(
    asset_id = "adv-missing-node-texture",
    asset_type = "texture",
    license = "CC0-1.0",
    seed = 99905,
    outputs = [output("textures/missing.png", "png", source = "nonexistent_node")],
    recipe = {
        "kind": "texture.procedural_v1",
        "params": texture_graph(
            [64, 64],
            [noise_node("actual_node", "simplex", 0.1, 4)]
        )
    }
)
