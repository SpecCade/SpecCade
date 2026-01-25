# Legacy jump animation spec
# Tests: position transforms, IK targets, non-looping

ANIMATION = {
    "name": "jump",
    "input_armature": "humanoid_basic_v1",
    "fps": 30,
    "duration_frames": 45,
    "loop": False,
    "ground_offset": 0.05,
    "poses": {
        "crouch": {
            "frame": 0,
            "bones": {
                "hips": {"position": [0.0, 0.0, -0.2]},
                "spine": {"pitch": 15.0},
                "chest": {"pitch": 10.0},
                "upper_leg_l": {"pitch": 60.0},
                "upper_leg_r": {"pitch": 60.0},
                "lower_leg_l": {"pitch": -120.0},
                "lower_leg_r": {"pitch": -120.0},
                "foot_l": {"pitch": 20.0},
                "foot_r": {"pitch": 20.0},
                "upper_arm_l": {"pitch": -20.0},
                "upper_arm_r": {"pitch": -20.0}
            }
        },
        "launch": {
            "frame": 10,
            "bones": {
                "hips": {"position": [0.0, 0.0, 0.0]},
                "spine": {"pitch": -10.0},
                "chest": {"pitch": -15.0},
                "upper_leg_l": {"pitch": -15.0},
                "upper_leg_r": {"pitch": -15.0},
                "lower_leg_l": {"pitch": -10.0},
                "lower_leg_r": {"pitch": -10.0},
                "foot_l": {"pitch": -30.0},
                "foot_r": {"pitch": -30.0},
                "upper_arm_l": {"pitch": -45.0},
                "upper_arm_r": {"pitch": -45.0}
            }
        },
        "apex": {
            "frame": 25,
            "bones": {
                "hips": {"position": [0.0, 0.0, 0.5]},
                "spine": {"pitch": -5.0},
                "chest": {"pitch": -5.0},
                "upper_leg_l": {"pitch": 20.0},
                "upper_leg_r": {"pitch": -10.0},
                "lower_leg_l": {"pitch": -30.0},
                "lower_leg_r": {"pitch": -60.0},
                "foot_l": {"pitch": 0.0},
                "foot_r": {"pitch": 10.0},
                "upper_arm_l": {"pitch": 0.0},
                "upper_arm_r": {"pitch": 0.0}
            }
        },
        "land": {
            "frame": 45,
            "bones": {
                "hips": {"position": [0.0, 0.0, -0.15]},
                "spine": {"pitch": 10.0},
                "chest": {"pitch": 5.0},
                "upper_leg_l": {"pitch": 45.0},
                "upper_leg_r": {"pitch": 45.0},
                "lower_leg_l": {"pitch": -90.0},
                "lower_leg_r": {"pitch": -90.0},
                "foot_l": {"pitch": 15.0},
                "foot_r": {"pitch": 15.0},
                "upper_arm_l": {"pitch": 20.0},
                "upper_arm_r": {"pitch": 20.0}
            }
        }
    },
    "phases": ["crouch", "launch", "apex", "land"],
    "ik_targets": {
        "foot_l": {
            "position": [0.1, 0.0, 0.0],
            "blend": 0.5
        },
        "foot_r": {
            "position": [-0.1, 0.0, 0.0],
            "blend": 0.5
        }
    },
    "save_blend": True
}
