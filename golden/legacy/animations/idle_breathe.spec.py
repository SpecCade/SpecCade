# Legacy .spec.py - REFERENCE ONLY (DO NOT EXECUTE)
# Idle breathing animation - subtle procedural movement

ANIMATION = {
    "name": "idle_breathe",
    "input_armature": "simple_biped.glb",
    "character": "simple_biped",
    "duration_frames": 60,
    "fps": 30,
    "loop": True,
    "ground_offset": 0.0,
    "poses": {
        "neutral": {
            "spine": {"pitch": 0, "yaw": 0, "roll": 0},
            "chest": {"pitch": 0, "yaw": 0, "roll": 0},
            "head": {"pitch": 0, "yaw": 0, "roll": 0}
        }
    },
    "phases": [
        {
            "name": "idle",
            "frames": [0, 60],
            "pose": "neutral",
            "timing_curve": "linear"
        }
    ],
    "procedural_layers": [
        {
            "type": "breathing",
            "target": "chest",
            "axis": "pitch",
            "period_frames": 60,
            "amplitude": 0.03
        },
        {
            "type": "sway",
            "target": "spine",
            "axis": "roll",
            "period_frames": 90,
            "amplitude": 0.01
        }
    ],
    "rig_setup": {
        "presets": {}
    },
    "save_blend": False
}
