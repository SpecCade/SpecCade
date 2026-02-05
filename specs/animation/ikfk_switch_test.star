# Verification test for IK/FK switching - arm switches from IK to FK mode

spec(
    asset_id = "ikfk_switch_test",
    asset_type = "skeletal_animation",
    license = "CC0-1.0",
    seed = 9005,
    description = "Verification test for IK/FK switching - arm switches from IK to FK mode",
    outputs = [output("ikfk_switch_test.glb", "glb")],
    recipe = {
        "kind": "skeletal_animation.blender_rigged_v1",
        "params": {
            "skeleton_preset": "humanoid_connected_v1",
            "clip_name": "ikfk_switch_test",
            "duration_frames": 90,
            "fps": 30,
            "loop": True,
            "rig_setup": {
                "presets": ["humanoid_arms"],
                "ikfk_switches": [
                    {
                        "name": "arm_l",
                        "ik_chain": "arm_l_ik",
                        "fk_bones": ["upper_arm_l", "lower_arm_l", "hand_l"],
                        "default_mode": "ik"
                    },
                    {
                        "name": "arm_r",
                        "ik_chain": "arm_r_ik",
                        "fk_bones": ["upper_arm_r", "lower_arm_r", "hand_r"],
                        "default_mode": "ik"
                    }
                ]
            },
            "ik_keyframes": [
                {
                    "time": 0.0,
                    "targets": {
                        "arm_l_ik_target": {"position": [0.5, 0.8, 1.2]},
                        "arm_r_ik_target": {"position": [-0.5, 0.8, 1.2]}
                    }
                },
                {
                    "time": 1.5,
                    "targets": {
                        "arm_l_ik_target": {"position": [0.3, 0.5, 1.0]},
                        "arm_r_ik_target": {"position": [-0.3, 0.5, 1.0]}
                    }
                },
                {
                    "time": 3.0,
                    "targets": {
                        "arm_l_ik_target": {"position": [0.5, 0.8, 1.2]},
                        "arm_r_ik_target": {"position": [-0.5, 0.8, 1.2]}
                    }
                }
            ],
            "save_blend": True,
            "preview": {
                "camera": "front",
                "size": [320, 320],
                "frame_step": 2,
                "background": [0.12, 0.15, 0.18, 1.0]
            }
        }
    }
)
