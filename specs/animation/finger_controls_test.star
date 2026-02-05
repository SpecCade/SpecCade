# Verification test for finger controls - hand opens and closes with curl/spread controls

spec(
    asset_id = "finger_controls_test",
    asset_type = "skeletal_animation",
    license = "CC0-1.0",
    seed = 9007,
    description = "Verification test for finger controls - hand opens and closes with curl/spread controls",
    outputs = [output("finger_controls_test.glb", "glb")],
    recipe = {
        "kind": "skeletal_animation.blender_rigged_v1",
        "params": {
            "skeleton_preset": "humanoid_connected_v1",
            "clip_name": "finger_controls_test",
            "duration_frames": 60,
            "fps": 30,
            "loop": True,
            "rig_setup": {
                "finger_controls": [
                    {
                        "name": "hand_l",
                        "side": "left",
                        "bone_prefix": "finger",
                        "fingers": ["thumb", "index", "middle", "ring", "pinky"],
                        "bones_per_finger": 3,
                        "max_curl_degrees": 90.0,
                        "max_spread_degrees": 15.0
                    },
                    {
                        "name": "hand_r",
                        "side": "right",
                        "bone_prefix": "finger",
                        "fingers": ["thumb", "index", "middle", "ring", "pinky"],
                        "bones_per_finger": 3,
                        "max_curl_degrees": 90.0,
                        "max_spread_degrees": 15.0
                    }
                ]
            },
            "keyframes": [
                {
                    "time": 0.0,
                    "bones": {
                        "upper_arm_l": {"rotation": [0, 0, -90]},
                        "upper_arm_r": {"rotation": [0, 0, 90]}
                    }
                },
                {
                    "time": 1.0,
                    "bones": {
                        "upper_arm_l": {"rotation": [0, 0, -90]},
                        "upper_arm_r": {"rotation": [0, 0, 90]}
                    }
                },
                {
                    "time": 2.0,
                    "bones": {
                        "upper_arm_l": {"rotation": [0, 0, -90]},
                        "upper_arm_r": {"rotation": [0, 0, 90]}
                    }
                }
            ],
            "finger_keyframes": [
                {"time": 0.0, "controls": "hand_l", "pose": {"curl": 0.0, "spread": 0.3}},
                {"time": 1.0, "controls": "hand_l", "pose": {"curl": 1.0, "spread": 0.0}},
                {"time": 2.0, "controls": "hand_l", "pose": {"curl": 0.0, "spread": 0.3}},
                {"time": 0.0, "controls": "hand_r", "pose": {"curl": 0.0, "spread": 0.3}},
                {"time": 1.0, "controls": "hand_r", "pose": {"curl": 1.0, "spread": 0.0, "index_curl": 0.0}},
                {"time": 2.0, "controls": "hand_r", "pose": {"curl": 0.0, "spread": 0.3}}
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
