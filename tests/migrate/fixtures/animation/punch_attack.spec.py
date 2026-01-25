# Legacy punch attack animation spec
# Tests: non-looping, yaw rotations, fast timing

ANIMATION = {
    "name": "punch",
    "rig": "humanoid",
    "fps": 60,
    "duration_frames": 30,
    "loop": False,
    "poses": {
        "windup": {
            "frame": 0,
            "bones": {
                "chest": {"yaw": 20.0, "pitch": 5.0},
                "spine": {"yaw": 10.0},
                "upper_arm_r": {"pitch": -30.0, "yaw": 45.0, "roll": 20.0},
                "lower_arm_r": {"pitch": -90.0},
                "hand_r": {"pitch": -15.0},
                "upper_arm_l": {"pitch": -15.0, "yaw": -10.0},
                "lower_arm_l": {"pitch": -45.0}
            }
        },
        "strike": {
            "frame": 10,
            "bones": {
                "chest": {"yaw": -10.0, "pitch": -5.0},
                "spine": {"yaw": -5.0},
                "upper_arm_r": {"pitch": 60.0, "yaw": 0.0, "roll": 0.0},
                "lower_arm_r": {"pitch": 0.0},
                "hand_r": {"pitch": 0.0},
                "upper_arm_l": {"pitch": 10.0, "yaw": 5.0},
                "lower_arm_l": {"pitch": -30.0}
            }
        },
        "followthrough": {
            "frame": 18,
            "bones": {
                "chest": {"yaw": -5.0, "pitch": 0.0},
                "spine": {"yaw": -2.0},
                "upper_arm_r": {"pitch": 30.0, "yaw": -10.0, "roll": -10.0},
                "lower_arm_r": {"pitch": -20.0},
                "hand_r": {"pitch": 10.0},
                "upper_arm_l": {"pitch": 5.0},
                "lower_arm_l": {"pitch": -35.0}
            }
        },
        "recover": {
            "frame": 30,
            "bones": {
                "chest": {"yaw": 0.0, "pitch": 0.0},
                "spine": {"yaw": 0.0},
                "upper_arm_r": {"pitch": 0.0, "yaw": 0.0, "roll": 0.0},
                "lower_arm_r": {"pitch": 0.0},
                "hand_r": {"pitch": 0.0},
                "upper_arm_l": {"pitch": 0.0, "yaw": 0.0},
                "lower_arm_l": {"pitch": 0.0}
            }
        }
    },
    "phases": ["windup", "strike", "followthrough", "recover"]
}
