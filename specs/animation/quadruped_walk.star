# Quadruped walk cycle - four-legged locomotion

spec(
    asset_id = "quadruped_walk",
    asset_type = "skeletal_animation",
    seed = 8008,
    license = "CC0-1.0",
    description = "Quadruped walk cycle - four-legged locomotion",
    outputs = [output("quadruped_walk.glb", "glb")],
    recipe = {
        "kind": "skeletal_animation.keyframe_v1",
        "params": {
            "skeleton_preset": "humanoid_connected_v1",
            "clip_name": "quadruped_walk",
            "duration_seconds": 1.33,
            "fps": 30,
            "loop": True,
            "interpolation": "bezier",
            "keyframes": [
                {
                    "time": 0.0,
                    "bones": {
                        "spine": {"rotation": [0, 0, -2]},
                        "upper_leg_l": {"rotation": [-15, 0, 0]},
                        "lower_leg_l": {"rotation": [10, 0, 0]},
                        "upper_leg_r": {"rotation": [20, 0, 0]},
                        "lower_leg_r": {"rotation": [-25, 0, 0]}
                    }
                },
                {
                    "time": 0.67,
                    "bones": {
                        "spine": {"rotation": [0, 0, 2]},
                        "upper_leg_l": {"rotation": [20, 0, 0]},
                        "lower_leg_l": {"rotation": [-25, 0, 0]},
                        "upper_leg_r": {"rotation": [-15, 0, 0]},
                        "lower_leg_r": {"rotation": [10, 0, 0]}
                    }
                },
                {
                    "time": 1.33,
                    "bones": {
                        "spine": {"rotation": [0, 0, -2]},
                        "upper_leg_l": {"rotation": [-15, 0, 0]},
                        "lower_leg_l": {"rotation": [10, 0, 0]},
                        "upper_leg_r": {"rotation": [20, 0, 0]},
                        "lower_leg_r": {"rotation": [-25, 0, 0]}
                    }
                }
            ]
        }
    }
)
