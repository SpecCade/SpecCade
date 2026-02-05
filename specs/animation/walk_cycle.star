# Walk cycle animation - basic bipedal locomotion

spec(
    asset_id = "walk_cycle",
    asset_type = "skeletal_animation",
    seed = 8002,
    license = "CC0-1.0",
    description = "Walk cycle animation - basic bipedal locomotion",
    outputs = [output("walk_cycle.glb", "glb")],
    recipe = {
        "kind": "skeletal_animation.keyframe_v1",
        "params": {
            "skeleton_preset": "humanoid_connected_v1",
            "clip_name": "walk_cycle",
            "duration_seconds": 1.0,
            "fps": 30,
            "loop": True,
            "interpolation": "bezier",
            "keyframes": [
                {
                    "time": 0.0,
                    "bones": {
                        "upper_leg_l": {"rotation": [-20, 0, 0]},
                        "lower_leg_l": {"rotation": [10, 0, 0]},
                        "upper_leg_r": {"rotation": [20, 0, 0]},
                        "lower_leg_r": {"rotation": [-30, 0, 0]},
                        "upper_arm_l": {"rotation": [15, 0, 0]},
                        "upper_arm_r": {"rotation": [-15, 0, 0]},
                        "spine": {"rotation": [0, 0, -3]}
                    }
                },
                {
                    "time": 0.25,
                    "bones": {
                        "upper_leg_l": {"rotation": [0, 0, 0]},
                        "lower_leg_l": {"rotation": [-40, 0, 0]},
                        "upper_leg_r": {"rotation": [0, 0, 0]},
                        "lower_leg_r": {"rotation": [0, 0, 0]},
                        "upper_arm_l": {"rotation": [0, 0, 0]},
                        "upper_arm_r": {"rotation": [0, 0, 0]},
                        "spine": {"rotation": [0, 0, 0]}
                    }
                },
                {
                    "time": 0.5,
                    "bones": {
                        "upper_leg_l": {"rotation": [20, 0, 0]},
                        "lower_leg_l": {"rotation": [-30, 0, 0]},
                        "upper_leg_r": {"rotation": [-20, 0, 0]},
                        "lower_leg_r": {"rotation": [10, 0, 0]},
                        "upper_arm_l": {"rotation": [-15, 0, 0]},
                        "upper_arm_r": {"rotation": [15, 0, 0]},
                        "spine": {"rotation": [0, 0, 3]}
                    }
                },
                {
                    "time": 0.75,
                    "bones": {
                        "upper_leg_l": {"rotation": [0, 0, 0]},
                        "lower_leg_l": {"rotation": [0, 0, 0]},
                        "upper_leg_r": {"rotation": [0, 0, 0]},
                        "lower_leg_r": {"rotation": [-40, 0, 0]},
                        "upper_arm_l": {"rotation": [0, 0, 0]},
                        "upper_arm_r": {"rotation": [0, 0, 0]},
                        "spine": {"rotation": [0, 0, 0]}
                    }
                },
                {
                    "time": 1.0,
                    "bones": {
                        "upper_leg_l": {"rotation": [-20, 0, 0]},
                        "lower_leg_l": {"rotation": [10, 0, 0]},
                        "upper_leg_r": {"rotation": [20, 0, 0]},
                        "lower_leg_r": {"rotation": [-30, 0, 0]},
                        "upper_arm_l": {"rotation": [15, 0, 0]},
                        "upper_arm_r": {"rotation": [-15, 0, 0]},
                        "spine": {"rotation": [0, 0, -3]}
                    }
                }
            ]
        }
    }
)
