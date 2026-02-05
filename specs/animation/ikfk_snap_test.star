# Verification test for IK/FK snapping - switches modes mid-animation with snapping

spec(
    asset_id = "ikfk_snap_test",
    asset_type = "skeletal_animation",
    license = "CC0-1.0",
    seed = 9006,
    description = "Test IK/FK snapping: arm moves with IK, switches to FK, poses manually, switches back to IK - all without pose pops",
    outputs = [output("ikfk_snap_test.glb", "glb")],
    recipe = {
        "kind": "skeletal_animation.blender_rigged_v1",
        "params": {
            "skeleton_preset": "humanoid_connected_v1",
            "clip_name": "ikfk_snap_test",
            "duration_frames": 120,
            "fps": 30,
            "loop": False,
            "rig_setup": {
                "presets": ["humanoid_arms"],
                "ikfk_switches": [
                    {
                        "name": "arm_l",
                        "ik_chain": "ik_arm_l",
                        "fk_bones": ["upper_arm_l", "lower_arm_l", "hand_l"],
                        "default_mode": "ik"
                    },
                    {
                        "name": "arm_r",
                        "ik_chain": "ik_arm_r",
                        "fk_bones": ["upper_arm_r", "lower_arm_r", "hand_r"],
                        "default_mode": "ik"
                    }
                ]
            },
            # IK keyframes: Phase 1 (0-1s) initial IK movement, Phase 5 (2.5-4s) final IK movement
            "ik_keyframes": [
                # Initial arm positions
                {
                    "time": 0.0,
                    "targets": {
                        "ik_hand_l": {"position": [0.7, 0.0, 1.35]},
                        "ik_hand_r": {"position": [-0.7, 0.0, 1.35]}
                    }
                },
                # Arms reach up (end of IK phase 1, before FK switch)
                {
                    "time": 1.0,
                    "targets": {
                        "ik_hand_l": {"position": [0.5, 0.5, 1.5]},
                        "ik_hand_r": {"position": [-0.5, 0.5, 1.5]}
                    }
                },
                # After FK phase, IK resumes (IK targets will snap to FK pose)
                {
                    "time": 3.0,
                    "targets": {
                        "ik_hand_l": {"position": [0.3, 0.3, 1.2]},
                        "ik_hand_r": {"position": [-0.3, 0.3, 1.2]}
                    }
                },
                # Final rest position
                {
                    "time": 4.0,
                    "targets": {
                        "ik_hand_l": {"position": [0.7, 0.0, 1.35]},
                        "ik_hand_r": {"position": [-0.7, 0.0, 1.35]}
                    }
                }
            ],
            # IK/FK mode switches with snapping
            "ikfk_keyframes": [
                # Switch to FK at 1s (snapping preserves IK pose in FK bones)
                {
                    "time": 1.0,
                    "switch": "arm_l",
                    "mode": "fk",
                    "snap": True
                },
                {
                    "time": 1.0,
                    "switch": "arm_r",
                    "mode": "fk",
                    "snap": True
                },
                # Switch back to IK at 2.5s (snapping moves IK targets to FK pose)
                {
                    "time": 2.5,
                    "switch": "arm_l",
                    "mode": "ik",
                    "snap": True
                },
                {
                    "time": 2.5,
                    "switch": "arm_r",
                    "mode": "ik",
                    "snap": True
                }
            ],
            # FK keyframes during FK mode (1-2.5s)
            "keyframes": [
                # Arms bend down during FK phase
                {
                    "time": 1.5,
                    "bones": {
                        "upper_arm_l": {"rotation": [0, 0, 30]},
                        "lower_arm_l": {"rotation": [45, 0, 0]},
                        "upper_arm_r": {"rotation": [0, 0, -30]},
                        "lower_arm_r": {"rotation": [45, 0, 0]}
                    }
                },
                {
                    "time": 2.0,
                    "bones": {
                        "upper_arm_l": {"rotation": [0, 0, 60]},
                        "lower_arm_l": {"rotation": [90, 0, 0]},
                        "upper_arm_r": {"rotation": [0, 0, -60]},
                        "lower_arm_r": {"rotation": [90, 0, 0]}
                    }
                }
            ],
            "save_blend": True,
            "export": {
                "include_armature": True,
                "include_animation": True,
                "bake_transforms": True
            },
            "preview": {
                "camera": "front",
                "size": [320, 320],
                "frame_step": 3,
                "background": [0.12, 0.15, 0.18, 1.0]
            }
        }
    }
)
