# Legacy walk cycle animation spec
# Tests: phases, poses, loop, basic bone rotations

ANIMATION = {
    "name": "walk_cycle",
    "rig": "humanoid",
    "fps": 30,
    "duration_frames": 60,
    "loop": True,
    "poses": {
        "contact_l": {
            "frame": 0,
            "bones": {
                "upper_leg_l": {"pitch": 20.0},
                "upper_leg_r": {"pitch": -15.0},
                "lower_leg_l": {"pitch": -30.0},
                "lower_leg_r": {"pitch": -5.0},
                "foot_l": {"pitch": 15.0},
                "foot_r": {"pitch": 0.0},
                "upper_arm_l": {"pitch": -15.0},
                "upper_arm_r": {"pitch": 10.0}
            }
        },
        "passing_r": {
            "frame": 15,
            "bones": {
                "upper_leg_l": {"pitch": 0.0},
                "upper_leg_r": {"pitch": 0.0},
                "lower_leg_l": {"pitch": 0.0},
                "lower_leg_r": {"pitch": -45.0},
                "foot_l": {"pitch": 0.0},
                "foot_r": {"pitch": 30.0},
                "upper_arm_l": {"pitch": 0.0},
                "upper_arm_r": {"pitch": 0.0}
            }
        },
        "contact_r": {
            "frame": 30,
            "bones": {
                "upper_leg_l": {"pitch": -15.0},
                "upper_leg_r": {"pitch": 20.0},
                "lower_leg_l": {"pitch": -5.0},
                "lower_leg_r": {"pitch": -30.0},
                "foot_l": {"pitch": 0.0},
                "foot_r": {"pitch": 15.0},
                "upper_arm_l": {"pitch": 10.0},
                "upper_arm_r": {"pitch": -15.0}
            }
        },
        "passing_l": {
            "frame": 45,
            "bones": {
                "upper_leg_l": {"pitch": 0.0},
                "upper_leg_r": {"pitch": 0.0},
                "lower_leg_l": {"pitch": -45.0},
                "lower_leg_r": {"pitch": 0.0},
                "foot_l": {"pitch": 30.0},
                "foot_r": {"pitch": 0.0},
                "upper_arm_l": {"pitch": 0.0},
                "upper_arm_r": {"pitch": 0.0}
            }
        }
    },
    "phases": ["contact_l", "passing_r", "contact_r", "passing_l"]
}
