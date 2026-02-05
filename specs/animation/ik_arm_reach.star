# Arm reach animation - reaching forward motion

spec(
    asset_id = "ik_arm_reach",
    asset_type = "skeletal_animation",
    seed = 8006,
    license = "CC0-1.0",
    description = "Arm reach animation - reaching forward motion",
    outputs = [output("ik_arm_reach.glb", "glb")],
    recipe = {
        "kind": "skeletal_animation.keyframe_v1",
        "params": {
            "skeleton_preset": "humanoid_connected_v1",
            "clip_name": "arm_reach",
            "duration_seconds": 2.0,
            "fps": 30,
            "loop": False,
            "interpolation": "bezier",
            "keyframes": [
                {
                    "time": 0.0,
                    "bones": {
                        "upper_arm_r": {"rotation": [0, 0, 0]},
                        "lower_arm_r": {"rotation": [0, 0, 0]},
                        "spine": {"rotation": [0, 0, 0]},
                        "head": {"rotation": [0, 0, 0]}
                    }
                },
                {
                    "time": 0.33,
                    "bones": {
                        "upper_arm_r": {"rotation": [-90, 20, 0]},
                        "lower_arm_r": {"rotation": [-15, 0, 0]},
                        "spine": {"rotation": [-10, 0, 0]},
                        "head": {"rotation": [-20, 15, 0]}
                    }
                },
                {
                    "time": 1.0,
                    "bones": {
                        "upper_arm_r": {"rotation": [-120, 30, 0]},
                        "lower_arm_r": {"rotation": [0, 0, 0]},
                        "spine": {"rotation": [-15, 0, 5]},
                        "head": {"rotation": [-30, 20, 0]}
                    }
                },
                {
                    "time": 2.0,
                    "bones": {
                        "upper_arm_r": {"rotation": [0, 0, 0]},
                        "lower_arm_r": {"rotation": [0, 0, 0]},
                        "spine": {"rotation": [0, 0, 0]},
                        "head": {"rotation": [0, 0, 0]}
                    }
                }
            ]
        }
    }
)
