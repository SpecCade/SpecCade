# Legacy idle breathing animation spec
# Tests: subtle movement, scale transforms, longer duration

ANIMATION = {
    "name": "idle_breathe",
    "skeleton": "humanoid_basic",
    "fps": 24,
    "duration_frames": 72,
    "loop": True,
    "poses": {
        "inhale": {
            "frame": 0,
            "bones": {
                "chest": {"scale": [1.0, 1.02, 1.0], "pitch": -2.0},
                "spine": {"pitch": -1.0},
                "shoulder_l": {"roll": 2.0},
                "shoulder_r": {"roll": -2.0},
                "head": {"pitch": -1.0}
            }
        },
        "peak": {
            "frame": 18,
            "bones": {
                "chest": {"scale": [1.0, 1.03, 1.0], "pitch": -3.0},
                "spine": {"pitch": -1.5},
                "shoulder_l": {"roll": 3.0},
                "shoulder_r": {"roll": -3.0},
                "head": {"pitch": -1.5}
            }
        },
        "exhale": {
            "frame": 36,
            "bones": {
                "chest": {"scale": [1.0, 1.0, 1.0], "pitch": 0.0},
                "spine": {"pitch": 0.5},
                "shoulder_l": {"roll": -1.0},
                "shoulder_r": {"roll": 1.0},
                "head": {"pitch": 0.5}
            }
        },
        "rest": {
            "frame": 54,
            "bones": {
                "chest": {"scale": [1.0, 0.99, 1.0], "pitch": 1.0},
                "spine": {"pitch": 1.0},
                "shoulder_l": {"roll": 0.0},
                "shoulder_r": {"roll": 0.0},
                "head": {"pitch": 0.0}
            }
        }
    },
    "phases": ["inhale", "peak", "exhale", "rest"]
}
