# Adversarial: Texture with circular node dependencies
# Expected: validation detects cycle or at least rejects the spec

spec(
    asset_id = "adv-circular-graph-texture",
    asset_type = "texture",
    license = "CC0-1.0",
    seed = 99906,
    outputs = [output("textures/circular.png", "png", source = "a")],
    recipe = {
        "kind": "texture.procedural_v1",
        "params": {
            "resolution": [64, 64],
            "tileable": False,
            "nodes": [
                {
                    "id": "a",
                    "type": "blend",
                    "inputs": ["b", "b"],
                    "mode": "multiply",
                    "factor": 0.5
                },
                {
                    "id": "b",
                    "type": "blend",
                    "inputs": ["a", "a"],
                    "mode": "multiply",
                    "factor": 0.5
                }
            ]
        }
    }
)
