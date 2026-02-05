# Verification test for bone constraints - hinge (elbow), ball (shoulder), planar (wrist)

spec(
    asset_id = "bone_constraints_test",
    asset_type = "skeletal_animation",
    license = "CC0-1.0",
    seed = 9004,
    description = "Verification test for bone constraints - hinge (elbow), ball (shoulder), planar (wrist)",
    outputs = [output("bone_constraints_test.glb", "glb")],
    recipe = {
        "kind": "skeletal_animation.blender_rigged_v1",
        "params": {
            "skeleton_preset": "humanoid_connected_v1",
            "clip_name": "bone_constraints_test",
            "duration_frames": 90,
            "fps": 30,
            "loop": True,
            "rig_setup": {
                "constraints": {
                    "constraints": [
                        {"type": "hinge", "bone": "lower_arm_l", "axis": "X", "min_angle": 0.0, "max_angle": 145.0},
                        {"type": "hinge", "bone": "lower_arm_r", "axis": "X", "min_angle": 0.0, "max_angle": 145.0},
                        {"type": "ball", "bone": "upper_arm_l", "cone_angle": 90.0, "twist_min": -90.0, "twist_max": 90.0},
                        {"type": "ball", "bone": "upper_arm_r", "cone_angle": 90.0, "twist_min": -90.0, "twist_max": 90.0},
                        {"type": "hinge", "bone": "lower_leg_l", "axis": "X", "min_angle": 0.0, "max_angle": 160.0},
                        {"type": "hinge", "bone": "lower_leg_r", "axis": "X", "min_angle": 0.0, "max_angle": 160.0}
                    ]
                }
            },
            "keyframes": [
                {
                    "time": 0.0,
                    "bones": {
                        "upper_arm_l": {"rotation": [0, 0, -90]},
                        "lower_arm_l": {"rotation": [0, 0, 0]},
                        "upper_arm_r": {"rotation": [0, 0, 90]},
                        "lower_arm_r": {"rotation": [0, 0, 0]},
                        "upper_leg_l": {"rotation": [0, 0, 0]},
                        "lower_leg_l": {"rotation": [0, 0, 0]},
                        "upper_leg_r": {"rotation": [0, 0, 0]},
                        "lower_leg_r": {"rotation": [0, 0, 0]}
                    }
                },
                {
                    "time": 1.5,
                    "bones": {
                        "upper_arm_l": {"rotation": [45, 30, -45]},
                        "lower_arm_l": {"rotation": [120, 0, 0]},
                        "upper_arm_r": {"rotation": [-45, -30, 45]},
                        "lower_arm_r": {"rotation": [120, 0, 0]},
                        "upper_leg_l": {"rotation": [-30, 0, 0]},
                        "lower_leg_l": {"rotation": [60, 0, 0]},
                        "upper_leg_r": {"rotation": [30, 0, 0]},
                        "lower_leg_r": {"rotation": [0, 0, 0]}
                    }
                },
                {
                    "time": 3.0,
                    "bones": {
                        "upper_arm_l": {"rotation": [0, 0, -90]},
                        "lower_arm_l": {"rotation": [0, 0, 0]},
                        "upper_arm_r": {"rotation": [0, 0, 90]},
                        "lower_arm_r": {"rotation": [0, 0, 0]},
                        "upper_leg_l": {"rotation": [0, 0, 0]},
                        "lower_leg_l": {"rotation": [0, 0, 0]},
                        "upper_leg_r": {"rotation": [0, 0, 0]},
                        "lower_leg_r": {"rotation": [0, 0, 0]}
                    }
                }
            ],
            "interpolation": "bezier",
            "save_blend": True,
            "preview": {
                "camera": "three_quarter",
                "size": [320, 320],
                "frame_step": 2,
                "background": [0.15, 0.17, 0.15, 1.0]
            }
        }
    }
)
