# Comprehensive skeletal animation fixture

spec(
    asset_id = "animation_comprehensive",
    asset_type = "skeletal_animation",
    seed = 4244,
    license = "CC0-1.0",
    description = "Comprehensive humanoid clip covering multiple bones and transform channels",
    outputs = [output("animations/animation_comprehensive.glb", "glb")],
    tags = ["golden", "comprehensive"],
    recipe = {
        "kind": "skeletal_animation.keyframe_v1",
        "params": {
            "skeleton_preset": "humanoid_connected_v1",
            "clip_name": "animation_comprehensive",
            "duration_seconds": 1.5,
            "fps": 24,
            "loop": True,
            "interpolation": "bezier",
            "keyframes": [
                {
                    "time": 0.0,
                    "bones": {
                        "root": {"position": [0.0, 0.0, 0.0], "rotation": [0.0, 0.0, 0.0]},
                        "spine": {"rotation": [2.0, 0.0, 0.0]},
                        "head": {"rotation": [0.0, 0.0, 0.0], "scale": [1.0, 1.0, 1.0]},
                        "upper_arm_l": {"rotation": [10.0, 0.0, -10.0]},
                        "lower_arm_l": {"rotation": [5.0, 0.0, 0.0]},
                        "upper_arm_r": {"rotation": [10.0, 0.0, 10.0]},
                        "lower_arm_r": {"rotation": [5.0, 0.0, 0.0]},
                        "upper_leg_l": {"rotation": [15.0, 0.0, 0.0]},
                        "lower_leg_l": {"rotation": [-10.0, 0.0, 0.0]},
                        "upper_leg_r": {"rotation": [-15.0, 0.0, 0.0]},
                        "lower_leg_r": {"rotation": [10.0, 0.0, 0.0]}
                    }
                },
                {
                    "time": 0.25,
                    "bones": {
                        "root": {"position": [0.0, 0.0, 0.0], "rotation": [0.0, 5.0, 0.0]},
                        "spine": {"rotation": [-4.0, 0.0, 0.0]},
                        "head": {"rotation": [0.0, 10.0, 0.0]},
                        "upper_arm_l": {"rotation": [-30.0, 15.0, -25.0]},
                        "lower_arm_l": {"rotation": [45.0, 0.0, 0.0]},
                        "upper_arm_r": {"rotation": [20.0, -10.0, 15.0]},
                        "lower_arm_r": {"rotation": [10.0, 0.0, 0.0]},
                        "upper_leg_l": {"rotation": [-25.0, 0.0, 0.0]},
                        "lower_leg_l": {"rotation": [20.0, 0.0, 0.0]},
                        "upper_leg_r": {"rotation": [25.0, 0.0, 0.0]},
                        "lower_leg_r": {"rotation": [-35.0, 0.0, 0.0]}
                    }
                },
                {
                    "time": 0.5,
                    "bones": {
                        "root": {"position": [0.0, 0.0, 0.0], "rotation": [0.0, 0.0, 0.0]},
                        "spine": {"rotation": [6.0, 0.0, 0.0]},
                        "head": {"rotation": [0.0, -10.0, 0.0], "scale": [1.02, 1.0, 1.02]},
                        "upper_arm_l": {"rotation": [20.0, -10.0, -15.0]},
                        "lower_arm_l": {"rotation": [10.0, 0.0, 0.0]},
                        "upper_arm_r": {"rotation": [-30.0, 15.0, 25.0]},
                        "lower_arm_r": {"rotation": [45.0, 0.0, 0.0]},
                        "upper_leg_l": {"rotation": [25.0, 0.0, 0.0]},
                        "lower_leg_l": {"rotation": [-35.0, 0.0, 0.0]},
                        "upper_leg_r": {"rotation": [-25.0, 0.0, 0.0]},
                        "lower_leg_r": {"rotation": [20.0, 0.0, 0.0]}
                    }
                },
                {
                    "time": 0.75,
                    "bones": {
                        "root": {"position": [0.0, 0.0, 0.0], "rotation": [0.0, -5.0, 0.0]},
                        "spine": {"rotation": [-4.0, 0.0, 0.0]},
                        "head": {"rotation": [0.0, 8.0, 0.0]},
                        "upper_arm_l": {"rotation": [-15.0, 10.0, -10.0]},
                        "lower_arm_l": {"rotation": [5.0, 0.0, 0.0]},
                        "upper_arm_r": {"rotation": [25.0, -15.0, 20.0]},
                        "lower_arm_r": {"rotation": [10.0, 0.0, 0.0]},
                        "upper_leg_l": {"rotation": [-15.0, 0.0, 0.0]},
                        "lower_leg_l": {"rotation": [10.0, 0.0, 0.0]},
                        "upper_leg_r": {"rotation": [15.0, 0.0, 0.0]},
                        "lower_leg_r": {"rotation": [-10.0, 0.0, 0.0]}
                    }
                },
                {
                    "time": 1.5,
                    "bones": {
                        "root": {"position": [0.0, 0.0, 0.0], "rotation": [0.0, 0.0, 0.0]},
                        "spine": {"rotation": [2.0, 0.0, 0.0]},
                        "head": {"rotation": [0.0, 0.0, 0.0], "scale": [1.0, 1.0, 1.0]},
                        "upper_arm_l": {"rotation": [10.0, 0.0, -10.0]},
                        "lower_arm_l": {"rotation": [5.0, 0.0, 0.0]},
                        "upper_arm_r": {"rotation": [10.0, 0.0, 10.0]},
                        "lower_arm_r": {"rotation": [5.0, 0.0, 0.0]},
                        "upper_leg_l": {"rotation": [15.0, 0.0, 0.0]},
                        "lower_leg_l": {"rotation": [-10.0, 0.0, 0.0]},
                        "upper_leg_r": {"rotation": [-15.0, 0.0, 0.0]},
                        "lower_leg_r": {"rotation": [10.0, 0.0, 0.0]}
                    }
                }
            ]
        }
    }
)
