# Idle breathing animation - subtle chest expansion

spec(
    asset_id = "idle_breathe",
    asset_type = "skeletal_animation",
    seed = 8001,
    license = "CC0-1.0",
    description = "Idle breathing animation - subtle chest expansion",
    outputs = [output("idle_breathe.glb", "glb")],
    recipe = {
        "kind": "skeletal_animation.keyframe_v1",
        "params": {
            "skeleton_preset": "humanoid_connected_v1",
            "clip_name": "idle_breathe",
            "duration_seconds": 2.0,
            "fps": 30,
            "loop": True,
            "interpolation": "bezier",
            "keyframes": [
                {
                    "time": 0.0,
                    "bones": {
                        "spine": {"rotation": [0, 0, 0]},
                        "chest": {"rotation": [0, 0, 0]},
                        "head": {"rotation": [0, 0, 0]}
                    }
                },
                {
                    "time": 1.0,
                    "bones": {
                        "spine": {"rotation": [1, 0, 0]},
                        "chest": {"rotation": [2, 0, 0]},
                        "head": {"rotation": [-1, 0, 0]}
                    }
                },
                {
                    "time": 2.0,
                    "bones": {
                        "spine": {"rotation": [0, 0, 0]},
                        "chest": {"rotation": [0, 0, 0]},
                        "head": {"rotation": [0, 0, 0]}
                    }
                }
            ]
        }
    }
)
