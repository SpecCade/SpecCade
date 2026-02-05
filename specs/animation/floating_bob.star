# Floating bob animation - gentle vertical oscillation

spec(
    asset_id = "floating_bob",
    asset_type = "skeletal_animation",
    seed = 8007,
    license = "CC0-1.0",
    description = "Floating bob animation - gentle vertical oscillation",
    outputs = [output("floating_bob.glb", "glb")],
    recipe = {
        "kind": "skeletal_animation.keyframe_v1",
        "params": {
            "skeleton_preset": "humanoid_connected_v1",
            "clip_name": "floating_bob",
            "duration_seconds": 3.0,
            "fps": 30,
            "loop": True,
            "interpolation": "bezier",
            "keyframes": [
                {
                    "time": 0.0,
                    "bones": {
                        "spine": {"rotation": [5, 0, 0]},
                        "upper_arm_l": {"rotation": [-30, 0, 15]},
                        "upper_arm_r": {"rotation": [-30, 0, -15]}
                    }
                },
                {
                    "time": 1.5,
                    "bones": {
                        "spine": {"rotation": [0, 0, 0]},
                        "upper_arm_l": {"rotation": [-35, 0, 20]},
                        "upper_arm_r": {"rotation": [-35, 0, -20]}
                    }
                },
                {
                    "time": 3.0,
                    "bones": {
                        "spine": {"rotation": [5, 0, 0]},
                        "upper_arm_l": {"rotation": [-30, 0, 15]},
                        "upper_arm_r": {"rotation": [-30, 0, -15]}
                    }
                }
            ]
        }
    }
)
