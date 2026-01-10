# Legacy .spec.py - REFERENCE ONLY (DO NOT EXECUTE)
# Simple beveled cube - basic primitive

MESH = {
    "name": "simple_cube",
    "primitive": "cube",
    "params": {
        "size": 1.0
    },
    "location": (0, 0, 0),
    "rotation": (0, 0, 0),
    "scale": (1, 1, 1),
    "shade": "smooth",
    "modifiers": [
        {
            "type": "bevel",
            "width": 0.02,
            "segments": 2
        }
    ],
    "uv": {
        "mode": "smart_project",
        "angle_limit": 66.0
    },
    "export": {
        "tangents": False
    }
}
