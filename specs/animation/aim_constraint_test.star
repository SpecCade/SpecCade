# Verification test for AimConstraint - head tracks a moving target

spec(
    asset_id = "aim_constraint_test",
    asset_type = "skeletal_animation",
    license = "CC0-1.0",
    seed = 9002,
    description = "Verification test for AimConstraint - head tracks a moving target",
    outputs = [output("aim_constraint_test.glb", "glb")],
    recipe = {
        "kind": "skeletal_animation.blender_rigged_v1",
        "params": {
            "skeleton_preset": "humanoid_connected_v1",
            "clip_name": "aim_constraint_test",
            "duration_frames": 90,
            "fps": 30,
            "loop": True,
            "rig_setup": {
                "aim_constraints": [
                    {
                        "name": "head_track",
                        "bone": "head",
                        "target": "look_target",
                        "track_axis": "Y",
                        "up_axis": "Z",
                        "influence": 0.8
                    }
                ]
            },
            "ik_keyframes": [
                {
                    "time": 0.0,
                    "targets": {
                        "look_target": {"position": [0.5, 2.0, 1.6]}
                    }
                },
                {
                    "time": 1.0,
                    "targets": {
                        "look_target": {"position": [-0.5, 2.0, 1.6]}
                    }
                },
                {
                    "time": 2.0,
                    "targets": {
                        "look_target": {"position": [0.0, 2.0, 1.8]}
                    }
                },
                {
                    "time": 3.0,
                    "targets": {
                        "look_target": {"position": [0.5, 2.0, 1.6]}
                    }
                }
            ],
            "procedural_layers": [
                {
                    "type": "sway",
                    "bone": "spine",
                    "axis": "yaw",
                    "amplitude": 2.0,
                    "period_frames": 90
                }
            ],
            "save_blend": True,
            "preview": {
                "camera": "front",
                "size": [320, 320],
                "frame_step": 2,
                "background": [0.12, 0.14, 0.18, 1.0]
            }
        }
    }
)
