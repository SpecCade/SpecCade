# Walk cycle animation using IK targets - demonstrates blender_rigged_v1 with foot IK and phases

spec(
    asset_id = "walk_cycle_ik",
    asset_type = "skeletal_animation",
    license = "CC0-1.0",
    seed = 8010,
    description = "Walk cycle animation using IK targets - demonstrates blender_rigged_v1 with foot IK and phases",
    outputs = [output("walk_cycle_ik.glb", "glb")],
    recipe = {
        "kind": "skeletal_animation.blender_rigged_v1",
        "params": {
            "skeleton_preset": "humanoid_connected_v1",
            "clip_name": "walk_cycle_ik",
            "duration_frames": 60,
            "fps": 30,
            "loop": True,
            "ground_offset": 0.0,
            "rig_setup": {
                "presets": ["humanoid_legs"],
                "ik_chains": [],
                "constraints": {
                    "constraints": []
                },
                "foot_systems": [],
                "aim_constraints": [],
                "twist_bones": []
            },
            "poses": {
                "contact_left": {
                    "bones": {
                        "spine": {"pitch": 2.0, "yaw": 0.0, "roll": -3.0},
                        "upper_arm_l": {"pitch": 15.0, "yaw": 0.0, "roll": 0.0},
                        "upper_arm_r": {"pitch": -15.0, "yaw": 0.0, "roll": 0.0}
                    }
                },
                "contact_right": {
                    "bones": {
                        "spine": {"pitch": 2.0, "yaw": 0.0, "roll": 3.0},
                        "upper_arm_l": {"pitch": -15.0, "yaw": 0.0, "roll": 0.0},
                        "upper_arm_r": {"pitch": 15.0, "yaw": 0.0, "roll": 0.0}
                    }
                }
            },
            "phases": [
                {
                    "name": "contact_left",
                    "start_frame": 1,
                    "end_frame": 15,
                    "curve": "ease_in_out",
                    "pose": "contact_left",
                    "ik_targets": {
                        "ik_leg_l": [
                            {"frame": 1, "location": [0.1, 0.3, 0.0]},
                            {"frame": 15, "location": [0.1, -0.1, 0.0]}
                        ],
                        "ik_leg_r": [
                            {"frame": 1, "location": [-0.1, -0.1, 0.0]},
                            {"frame": 15, "location": [-0.1, 0.2, 0.15]}
                        ]
                    }
                },
                {
                    "name": "passing_left",
                    "start_frame": 15,
                    "end_frame": 30,
                    "curve": "linear"
                },
                {
                    "name": "contact_right",
                    "start_frame": 30,
                    "end_frame": 45,
                    "curve": "ease_in_out",
                    "pose": "contact_right",
                    "ik_targets": {
                        "ik_leg_l": [
                            {"frame": 30, "location": [0.1, -0.1, 0.0]},
                            {"frame": 45, "location": [0.1, 0.2, 0.15]}
                        ],
                        "ik_leg_r": [
                            {"frame": 30, "location": [-0.1, 0.3, 0.0]},
                            {"frame": 45, "location": [-0.1, -0.1, 0.0]}
                        ]
                    }
                },
                {
                    "name": "passing_right",
                    "start_frame": 45,
                    "end_frame": 60,
                    "curve": "linear"
                }
            ],
            "keyframes": [],
            "ik_keyframes": [],
            "interpolation": "bezier",
            "save_blend": True
        }
    }
)
