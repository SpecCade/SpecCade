# Legacy .spec.py - REFERENCE ONLY (DO NOT EXECUTE)
# Cylinder with beveled edges - good for pillars

MESH = {
    "name": "cylinder_capped",
    "primitive": "cylinder",
    "params": {
        "radius": 0.3,
        "depth": 2.0,
        "vertices": 32
    },
    "location": (0, 0, 0),
    "rotation": (0, 0, 0),
    "scale": (1, 1, 1),
    "shade": "smooth",
    "modifiers": [
        {
            "type": "bevel",
            "width": 0.03,
            "segments": 3,
            "angle_limit": 0.5236
        }
    ],
    "uv": {
        "mode": "cube_project",
        "cube_size": 1.0
    },
    "export": {
        "tangents": False
    }
}
