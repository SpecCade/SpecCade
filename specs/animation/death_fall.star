# Death fall animation - ragdoll-like collapse

spec(
    asset_id = "death_fall",
    asset_type = "skeletal_animation",
    seed = 8005,
    license = "CC0-1.0",
    description = "Death fall animation - ragdoll-like collapse",
    outputs = [output("death_fall.glb", "glb")],
    recipe = {
        "kind": "skeletal_animation.keyframe_v1",
        "params": {
            "skeleton_preset": "humanoid_connected_v1",
            "clip_name": "death_fall",
            "duration_seconds": 1.5,
            "fps": 30,
            "loop": False,
            "interpolation": "bezier",
            "keyframes": [
                {
                    "time": 0.0,
                    "bones": {
                        "spine": {"rotation": [0, 0, 0]},
                        "head": {"rotation": [0, 0, 0]}
                    }
                },
                {
                    "time": 0.2,
                    "bones": {
                        "spine": {"rotation": [-15, 0, 10]},
                        "head": {"rotation": [20, -30, 0]}
                    }
                },
                {
                    "time": 0.8,
                    "bones": {
                        "spine": {"rotation": [-45, 0, 20]},
                        "head": {"rotation": [40, -45, 20]}
                    }
                },
                {
                    "time": 1.5,
                    "bones": {
                        "spine": {"rotation": [-80, 0, 30]},
                        "head": {"rotation": [60, -60, 30]}
                    }
                }
            ]
        }
    }
)
