# Verification test for TwistBone - arm rotation distributed across twist bones

spec(
    asset_id = "twist_bones_test",
    asset_type = "skeletal_animation",
    license = "CC0-1.0",
    seed = 9003,
    description = "Verification test for TwistBone - arm rotation distributed across twist bones (uses humanoid_connected_v1 with built-in twist bones)",
    outputs = [output("twist_bones_test.glb", "glb")],
    recipe = {
        "kind": "skeletal_animation.blender_rigged_v1",
        "params": {
            "skeleton_preset": "humanoid_connected_v1",
            "clip_name": "twist_bones_test",
            "duration_frames": 60,
            "fps": 30,
            "loop": True,
            "rig_setup": {
                "presets": ["humanoid_arms"],
                "twist_bones": [
                    {
                        "name": "upper_arm_twist_setup_l",
                        "source": "upper_arm_l",
                        "target": "upper_arm_twist_l",
                        "axis": "Y",
                        "influence": 0.5
                    },
                    {
                        "name": "lower_arm_twist_setup_l",
                        "source": "lower_arm_l",
                        "target": "lower_arm_twist_l",
                        "axis": "Y",
                        "influence": 0.5
                    },
                    {
                        "name": "upper_arm_twist_setup_r",
                        "source": "upper_arm_r",
                        "target": "upper_arm_twist_r",
                        "axis": "Y",
                        "influence": 0.5
                    },
                    {
                        "name": "lower_arm_twist_setup_r",
                        "source": "lower_arm_r",
                        "target": "lower_arm_twist_r",
                        "axis": "Y",
                        "influence": 0.5
                    }
                ]
            },
            "keyframes": [
                {
                    "time": 0.0,
                    "bones": {
                        "upper_arm_l": {"rotation": [0, 0, -80]},
                        "lower_arm_l": {"rotation": [0, 0, 0]},
                        "upper_arm_r": {"rotation": [0, 0, 80]},
                        "lower_arm_r": {"rotation": [0, 0, 0]}
                    }
                },
                {
                    "time": 1.0,
                    "bones": {
                        "upper_arm_l": {"rotation": [0, -90, -80]},
                        "lower_arm_l": {"rotation": [0, -90, 0]},
                        "upper_arm_r": {"rotation": [0, 90, 80]},
                        "lower_arm_r": {"rotation": [0, 90, 0]}
                    }
                },
                {
                    "time": 2.0,
                    "bones": {
                        "upper_arm_l": {"rotation": [0, 0, -80]},
                        "lower_arm_l": {"rotation": [0, 0, 0]},
                        "upper_arm_r": {"rotation": [0, 0, 80]},
                        "lower_arm_r": {"rotation": [0, 0, 0]}
                    }
                }
            ],
            "save_blend": True,
            "preview": {
                "camera": "front",
                "size": [320, 320],
                "frame_step": 2,
                "background": [0.18, 0.15, 0.12, 1.0]
            }
        }
    }
)
