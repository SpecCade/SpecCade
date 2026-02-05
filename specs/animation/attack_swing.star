# Attack swing animation - simple overhead swing

spec(
    asset_id = "attack_swing",
    asset_type = "skeletal_animation",
    seed = 8003,
    license = "CC0-1.0",
    description = "Attack swing animation - simple overhead swing",
    outputs = [output("attack_swing.glb", "glb")],
    recipe = {
        "kind": "skeletal_animation.keyframe_v1",
        "params": {
            "skeleton_preset": "humanoid_connected_v1",
            "clip_name": "attack_swing",
            "duration_seconds": 0.8,
            "fps": 30,
            "loop": False,
            "interpolation": "bezier",
            "keyframes": [
                {
                    "time": 0.0,
                    "bones": {
                        "upper_arm_r": {"rotation": [-60, 30, 0]},
                        "lower_arm_r": {"rotation": [-90, 0, 0]},
                        "spine": {"rotation": [10, -20, 0]}
                    }
                },
                {
                    "time": 0.33,
                    "bones": {
                        "upper_arm_r": {"rotation": [-120, 45, 0]},
                        "lower_arm_r": {"rotation": [-45, 0, 0]},
                        "spine": {"rotation": [15, -30, -5]}
                    }
                },
                {
                    "time": 0.47,
                    "bones": {
                        "upper_arm_r": {"rotation": [30, -10, 0]},
                        "lower_arm_r": {"rotation": [-15, 0, 0]},
                        "spine": {"rotation": [-10, 20, 5]}
                    }
                },
                {
                    "time": 0.8,
                    "bones": {
                        "upper_arm_r": {"rotation": [45, -20, 0]},
                        "lower_arm_r": {"rotation": [-30, 0, 0]},
                        "spine": {"rotation": [-5, 10, 3]}
                    }
                }
            ]
        }
    }
)
