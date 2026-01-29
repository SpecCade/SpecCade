# Verification test for FootSystem - heel/ball/toe roll pivots during walk

spec(
    asset_id = "foot_roll_test",
    asset_type = "skeletal_animation",
    license = "CC0-1.0",
    seed = 9001,
    description = "Verification test for FootSystem - heel/ball/toe roll pivots during walk",
    outputs = [output("foot_roll_test.glb", "glb")],
    recipe = {
        "kind": "skeletal_animation.blender_rigged_v1",
        "params": {
            "skeleton_preset": "humanoid_basic_v1",
            "clip_name": "foot_roll_test",
            "duration_frames": 60,
            "fps": 30,
            "loop": True,
            "rig_setup": {
                "presets": ["humanoid_legs"],
                "foot_systems": [
                    {
                        "name": "foot_roll_l",
                        "foot_bone": "foot_l",
                        "toe_bone": "foot_l",
                        "roll_limits": [-30.0, 60.0]
                    },
                    {
                        "name": "foot_roll_r",
                        "foot_bone": "foot_r",
                        "toe_bone": "foot_r",
                        "roll_limits": [-30.0, 60.0]
                    }
                ]
            },
            "phases": [
                {
                    "name": "heel_strike_left",
                    "start_frame": 1,
                    "end_frame": 15,
                    "curve": "ease_out",
                    "ik_targets": {
                        "ik_leg_l": [
                            {"frame": 1, "location": [0.1, 0.25, 0.02]},
                            {"frame": 15, "location": [0.1, 0.1, 0.0]}
                        ],
                        "ik_leg_r": [
                            {"frame": 1, "location": [-0.1, -0.1, 0.0]},
                            {"frame": 15, "location": [-0.1, -0.2, 0.08]}
                        ]
                    }
                },
                {
                    "name": "toe_off_left",
                    "start_frame": 15,
                    "end_frame": 30,
                    "curve": "ease_in",
                    "ik_targets": {
                        "ik_leg_l": [
                            {"frame": 15, "location": [0.1, 0.1, 0.0]},
                            {"frame": 30, "location": [0.1, -0.2, 0.12]}
                        ],
                        "ik_leg_r": [
                            {"frame": 15, "location": [-0.1, -0.2, 0.08]},
                            {"frame": 30, "location": [-0.1, 0.25, 0.02]}
                        ]
                    }
                },
                {
                    "name": "heel_strike_right",
                    "start_frame": 30,
                    "end_frame": 45,
                    "curve": "ease_out",
                    "ik_targets": {
                        "ik_leg_l": [
                            {"frame": 30, "location": [0.1, -0.2, 0.12]},
                            {"frame": 45, "location": [0.1, -0.1, 0.0]}
                        ],
                        "ik_leg_r": [
                            {"frame": 30, "location": [-0.1, 0.25, 0.02]},
                            {"frame": 45, "location": [-0.1, 0.1, 0.0]}
                        ]
                    }
                },
                {
                    "name": "toe_off_right",
                    "start_frame": 45,
                    "end_frame": 60,
                    "curve": "ease_in",
                    "ik_targets": {
                        "ik_leg_l": [
                            {"frame": 45, "location": [0.1, -0.1, 0.0]},
                            {"frame": 60, "location": [0.1, 0.25, 0.02]}
                        ],
                        "ik_leg_r": [
                            {"frame": 45, "location": [-0.1, 0.1, 0.0]},
                            {"frame": 60, "location": [-0.1, -0.2, 0.12]}
                        ]
                    }
                }
            ],
            "save_blend": True,
            "preview": {
                "camera": "side",
                "size": [320, 320],
                "frame_step": 2,
                "background": [0.15, 0.15, 0.18, 1.0]
            }
        }
    }
)
