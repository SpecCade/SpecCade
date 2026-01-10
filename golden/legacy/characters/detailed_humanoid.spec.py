# Legacy .spec.py - REFERENCE ONLY (DO NOT EXECUTE)
# Detailed humanoid - more complex skeleton with fingers

SPEC = {
    "name": "detailed_humanoid",
    "tri_budget": 2000,
    "skeleton": [
        {"bone": "root", "head": [0, 0, 0], "tail": [0, 0, 0.1]},
        {"bone": "spine_01", "head": [0, 0, 0.1], "tail": [0, 0, 0.3], "parent": "root"},
        {"bone": "spine_02", "head": [0, 0, 0.3], "tail": [0, 0, 0.5], "parent": "spine_01"},
        {"bone": "spine_03", "head": [0, 0, 0.5], "tail": [0, 0, 0.7], "parent": "spine_02"},
        {"bone": "chest", "head": [0, 0, 0.7], "tail": [0, 0, 0.85], "parent": "spine_03"},
        {"bone": "neck", "head": [0, 0, 0.85], "tail": [0, 0, 0.95], "parent": "chest"},
        {"bone": "head", "head": [0, 0, 0.95], "tail": [0, 0, 1.15], "parent": "neck"},

        {"bone": "shoulder_L", "head": [0.05, 0, 0.8], "tail": [0.15, 0, 0.8], "parent": "chest"},
        {"bone": "arm_upper_L", "head": [0.15, 0, 0.8], "tail": [0.4, 0, 0.8], "parent": "shoulder_L"},
        {"bone": "arm_lower_L", "head": [0.4, 0, 0.8], "tail": [0.65, 0, 0.8], "parent": "arm_upper_L"},
        {"bone": "hand_L", "head": [0.65, 0, 0.8], "tail": [0.75, 0, 0.8], "parent": "arm_lower_L"},

        {"bone": "finger_index_01_L", "head": [0.75, 0.02, 0.8], "tail": [0.78, 0.02, 0.8], "parent": "hand_L"},
        {"bone": "finger_index_02_L", "head": [0.78, 0.02, 0.8], "tail": [0.81, 0.02, 0.8], "parent": "finger_index_01_L"},
        {"bone": "finger_middle_01_L", "head": [0.75, 0, 0.8], "tail": [0.79, 0, 0.8], "parent": "hand_L"},
        {"bone": "finger_middle_02_L", "head": [0.79, 0, 0.8], "tail": [0.83, 0, 0.8], "parent": "finger_middle_01_L"},
        {"bone": "thumb_01_L", "head": [0.68, 0.05, 0.78], "tail": [0.71, 0.07, 0.77], "parent": "hand_L"},
        {"bone": "thumb_02_L", "head": [0.71, 0.07, 0.77], "tail": [0.74, 0.09, 0.76], "parent": "thumb_01_L"},

        {"bone": "shoulder_R", "mirror": "shoulder_L"},
        {"bone": "arm_upper_R", "mirror": "arm_upper_L"},
        {"bone": "arm_lower_R", "mirror": "arm_lower_L"},
        {"bone": "hand_R", "mirror": "hand_L"},
        {"bone": "finger_index_01_R", "mirror": "finger_index_01_L"},
        {"bone": "finger_index_02_R", "mirror": "finger_index_02_L"},
        {"bone": "finger_middle_01_R", "mirror": "finger_middle_01_L"},
        {"bone": "finger_middle_02_R", "mirror": "finger_middle_02_L"},
        {"bone": "thumb_01_R", "mirror": "thumb_01_L"},
        {"bone": "thumb_02_R", "mirror": "thumb_02_L"},

        {"bone": "hip_L", "head": [0.08, 0, 0.1], "tail": [0.12, 0, 0.05], "parent": "root"},
        {"bone": "leg_upper_L", "head": [0.12, 0, 0.05], "tail": [0.12, 0, -0.4], "parent": "hip_L"},
        {"bone": "leg_lower_L", "head": [0.12, 0, -0.4], "tail": [0.12, 0, -0.8], "parent": "leg_upper_L"},
        {"bone": "foot_L", "head": [0.12, 0, -0.8], "tail": [0.12, 0.12, -0.8], "parent": "leg_lower_L"},
        {"bone": "toe_L", "head": [0.12, 0.12, -0.8], "tail": [0.12, 0.18, -0.8], "parent": "foot_L"},

        {"bone": "hip_R", "mirror": "hip_L"},
        {"bone": "leg_upper_R", "mirror": "leg_upper_L"},
        {"bone": "leg_lower_R", "mirror": "leg_lower_L"},
        {"bone": "foot_R", "mirror": "foot_L"},
        {"bone": "toe_R", "mirror": "toe_L"}
    ],
    "parts": {
        "torso": {
            "bone": "spine_01",
            "base": "hexagon(8)",
            "base_radius": [0.12, 0.14],
            "steps": [
                {"extrude": 0.2, "scale": 1.1},
                {"extrude": 0.2, "scale": 1.2},
                {"extrude": 0.15, "scale": [1.3, 1.0]},
                {"extrude": 0.15, "scale": 1.1}
            ],
            "cap_start": True,
            "cap_end": True,
            "skinning_type": "soft"
        },
        "head_mesh": {
            "bone": "head",
            "base": "hexagon(10)",
            "base_radius": 0.1,
            "offset": [0, 0, 0.95],
            "steps": [
                {"extrude": 0.08, "scale": 1.2},
                {"extrude": 0.12, "scale": 1.0},
                {"extrude": 0.05, "scale": 0.9}
            ],
            "cap_start": True,
            "cap_end": True
        },
        "arm_L": {
            "bone": "arm_upper_L",
            "base": "hexagon(6)",
            "base_radius": [0.05, 0.04],
            "offset": [0.15, 0, 0.8],
            "rotation": [0, 90, 0],
            "steps": [
                {"extrude": 0.25, "scale": 0.85},
                {"extrude": 0.25, "scale": 0.75},
                {"extrude": 0.1, "scale": 0.6}
            ],
            "skinning_type": "soft"
        },
        "arm_R": {"mirror": "arm_L"},
        "leg_L": {
            "bone": "leg_upper_L",
            "base": "hexagon(6)",
            "base_radius": [0.08, 0.06],
            "offset": [0.12, 0, 0.05],
            "rotation": [180, 0, 0],
            "steps": [
                {"extrude": 0.45, "scale": 0.8},
                {"extrude": 0.4, "scale": 0.6},
                {"extrude": 0.05, "scale": 0.9}
            ],
            "skinning_type": "soft"
        },
        "leg_R": {"mirror": "leg_L"}
    },
    "texturing": {
        "uv_mode": "region_based",
        "regions": {
            "body": {"parts": ["torso"], "uv_scale": 1.0},
            "limbs": {"parts": ["arm_L", "arm_R", "leg_L", "leg_R"], "uv_scale": 0.8},
            "head": {"parts": ["head_mesh"], "uv_scale": 1.2}
        }
    }
}
