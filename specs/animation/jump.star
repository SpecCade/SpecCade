# Jump animation - anticipation, launch, apex, landing

spec(
    asset_id = "jump",
    asset_type = "skeletal_animation",
    seed = 8004,
    license = "CC0-1.0",
    description = "Jump animation - anticipation, launch, apex, landing",
    outputs = [output("jump.glb", "glb")],
    recipe = {
        "kind": "skeletal_animation.keyframe_v1",
        "params": {
            "skeleton_preset": "humanoid_connected_v1",
            "clip_name": "jump",
            "duration_seconds": 1.2,
            "fps": 30,
            "loop": False,
            "interpolation": "bezier",
            "keyframes": [
                {
                    "time": 0.0,
                    "bones": {
                        "spine": {"rotation": [20, 0, 0]},
                        "upper_leg_l": {"rotation": [45, 0, 0]},
                        "lower_leg_l": {"rotation": [-90, 0, 0]},
                        "upper_leg_r": {"rotation": [45, 0, 0]},
                        "lower_leg_r": {"rotation": [-90, 0, 0]}
                    }
                },
                {
                    "time": 0.27,
                    "bones": {
                        "spine": {"rotation": [-10, 0, 0]},
                        "upper_leg_l": {"rotation": [-15, 0, 0]},
                        "lower_leg_l": {"rotation": [0, 0, 0]},
                        "upper_leg_r": {"rotation": [-15, 0, 0]},
                        "lower_leg_r": {"rotation": [0, 0, 0]}
                    }
                },
                {
                    "time": 0.47,
                    "bones": {
                        "spine": {"rotation": [0, 0, 0]},
                        "upper_leg_l": {"rotation": [10, 0, 0]},
                        "lower_leg_l": {"rotation": [-20, 0, 0]},
                        "upper_leg_r": {"rotation": [10, 0, 0]},
                        "lower_leg_r": {"rotation": [-20, 0, 0]}
                    }
                },
                {
                    "time": 1.2,
                    "bones": {
                        "spine": {"rotation": [15, 0, 0]},
                        "upper_leg_l": {"rotation": [30, 0, 0]},
                        "lower_leg_l": {"rotation": [-60, 0, 0]},
                        "upper_leg_r": {"rotation": [30, 0, 0]},
                        "lower_leg_r": {"rotation": [-60, 0, 0]}
                    }
                }
            ]
        }
    }
)
