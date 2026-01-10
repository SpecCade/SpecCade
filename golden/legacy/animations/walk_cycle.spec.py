# Legacy .spec.py - REFERENCE ONLY (DO NOT EXECUTE)
# Walk cycle animation - basic locomotion

ANIMATION = {
    "name": "walk_cycle",
    "input_armature": "simple_biped.glb",
    "character": "simple_biped",
    "duration_frames": 30,
    "fps": 30,
    "loop": True,
    "ground_offset": 0.0,
    "poses": {
        "contact_L": {
            "leg_upper_L": {"pitch": -20},
            "leg_lower_L": {"pitch": 10},
            "leg_upper_R": {"pitch": 20},
            "leg_lower_R": {"pitch": -30},
            "arm_upper_L": {"pitch": 15},
            "arm_upper_R": {"pitch": -15},
            "spine": {"roll": -3}
        },
        "passing_L": {
            "leg_upper_L": {"pitch": 0},
            "leg_lower_L": {"pitch": -40},
            "leg_upper_R": {"pitch": 0},
            "leg_lower_R": {"pitch": 0},
            "arm_upper_L": {"pitch": 0},
            "arm_upper_R": {"pitch": 0},
            "spine": {"roll": 0}
        },
        "contact_R": {
            "leg_upper_L": {"pitch": 20},
            "leg_lower_L": {"pitch": -30},
            "leg_upper_R": {"pitch": -20},
            "leg_lower_R": {"pitch": 10},
            "arm_upper_L": {"pitch": -15},
            "arm_upper_R": {"pitch": 15},
            "spine": {"roll": 3}
        },
        "passing_R": {
            "leg_upper_L": {"pitch": 0},
            "leg_lower_L": {"pitch": 0},
            "leg_upper_R": {"pitch": 0},
            "leg_lower_R": {"pitch": -40},
            "arm_upper_L": {"pitch": 0},
            "arm_upper_R": {"pitch": 0},
            "spine": {"roll": 0}
        }
    },
    "phases": [
        {"name": "contact_L", "frames": [0, 7], "pose": "contact_L", "timing_curve": "ease_in_out"},
        {"name": "passing_L", "frames": [7, 15], "pose": "passing_L", "timing_curve": "ease_in_out"},
        {"name": "contact_R", "frames": [15, 22], "pose": "contact_R", "timing_curve": "ease_in_out"},
        {"name": "passing_R", "frames": [22, 30], "pose": "passing_R", "timing_curve": "ease_in_out"}
    ],
    "rig_setup": {
        "presets": {
            "humanoid_legs": True
        }
    },
    "save_blend": False
}
