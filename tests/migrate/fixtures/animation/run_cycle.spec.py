# Legacy run cycle animation spec
# Tests: faster timing, more extreme poses

ANIMATION = {
    "name": "run_cycle",
    "character": "humanoid_basic_v1",
    "fps": 30,
    "duration_frames": 20,
    "loop": True,
    "poses": {
        "flight_l": {
            "frame": 0,
            "bones": {
                "hips": {"position": [0.0, 0.0, 0.1]},
                "upper_leg_l": {"pitch": 45.0},
                "upper_leg_r": {"pitch": -30.0},
                "lower_leg_l": {"pitch": -90.0},
                "lower_leg_r": {"pitch": -45.0},
                "foot_l": {"pitch": -20.0},
                "foot_r": {"pitch": 15.0},
                "upper_arm_l": {"pitch": -30.0},
                "upper_arm_r": {"pitch": 45.0},
                "lower_arm_l": {"pitch": -15.0},
                "lower_arm_r": {"pitch": -30.0}
            }
        },
        "contact_r": {
            "frame": 5,
            "bones": {
                "hips": {"position": [0.0, 0.0, 0.0]},
                "upper_leg_l": {"pitch": -20.0},
                "upper_leg_r": {"pitch": 10.0},
                "lower_leg_l": {"pitch": -10.0},
                "lower_leg_r": {"pitch": -30.0},
                "foot_l": {"pitch": 0.0},
                "foot_r": {"pitch": 20.0},
                "upper_arm_l": {"pitch": 15.0},
                "upper_arm_r": {"pitch": -20.0}
            }
        },
        "flight_r": {
            "frame": 10,
            "bones": {
                "hips": {"position": [0.0, 0.0, 0.1]},
                "upper_leg_l": {"pitch": -30.0},
                "upper_leg_r": {"pitch": 45.0},
                "lower_leg_l": {"pitch": -45.0},
                "lower_leg_r": {"pitch": -90.0},
                "foot_l": {"pitch": 15.0},
                "foot_r": {"pitch": -20.0},
                "upper_arm_l": {"pitch": 45.0},
                "upper_arm_r": {"pitch": -30.0},
                "lower_arm_l": {"pitch": -30.0},
                "lower_arm_r": {"pitch": -15.0}
            }
        },
        "contact_l": {
            "frame": 15,
            "bones": {
                "hips": {"position": [0.0, 0.0, 0.0]},
                "upper_leg_l": {"pitch": 10.0},
                "upper_leg_r": {"pitch": -20.0},
                "lower_leg_l": {"pitch": -30.0},
                "lower_leg_r": {"pitch": -10.0},
                "foot_l": {"pitch": 20.0},
                "foot_r": {"pitch": 0.0},
                "upper_arm_l": {"pitch": -20.0},
                "upper_arm_r": {"pitch": 15.0}
            }
        }
    },
    "phases": ["flight_l", "contact_r", "flight_r", "contact_l"]
}
