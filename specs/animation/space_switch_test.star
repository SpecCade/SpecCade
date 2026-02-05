# Verification test for space switching - hand switches between world and chest space

spec(
    asset_id = "space_switch_test",
    asset_type = "skeletal_animation",
    license = "CC0-1.0",
    seed = 9006,
    description = "Verification test for space switching - hand switches between world and chest space",
    outputs = [output("space_switch_test.glb", "glb")],
    recipe = {
        "kind": "skeletal_animation.blender_rigged_v1",
        "params": {
            "skeleton_preset": "humanoid_connected_v1",
            "clip_name": "space_switch_test",
            "duration_frames": 90,
            "fps": 30,
            "loop": True,
            "rig_setup": {
                "presets": ["humanoid_arms"],
                "space_switches": [
                    {
                        "name": "hand_l_space",
                        "bone": "hand_l",
                        "spaces": [
                            {"name": "World", "type": "world"},
                            {"name": "Root", "type": "root"},
                            {"name": "Chest", "type": "bone", "bone": "chest"}
                        ],
                        "default_space": 0
                    }
                ]
            },
            "keyframes": [
                {
                    "time": 0.0,
                    "bones": {
                        "upper_arm_l": {"rotation": [0, 0, -45]},
                        "lower_arm_l": {"rotation": [45, 0, 0]}
                    }
                },
                {
                    "time": 1.5,
                    "bones": {
                        "upper_arm_l": {"rotation": [30, 30, -60]},
                        "lower_arm_l": {"rotation": [90, 0, 0]},
                        "chest": {"rotation": [0, 20, 0]}
                    }
                },
                {
                    "time": 3.0,
                    "bones": {
                        "upper_arm_l": {"rotation": [0, 0, -45]},
                        "lower_arm_l": {"rotation": [45, 0, 0]}
                    }
                }
            ],
            "save_blend": True,
            "preview": {
                "camera": "three_quarter",
                "size": [320, 320],
                "frame_step": 2,
                "background": [0.15, 0.12, 0.18, 1.0]
            }
        }
    }
)
