# Legacy .spec.py - REFERENCE ONLY (DO NOT EXECUTE)
# Low-poly icosphere with decimation - game-ready

MESH = {
    "name": "lowpoly_icosphere",
    "primitive": "icosphere",
    "params": {
        "radius": 0.5,
        "subdivisions": 2
    },
    "location": (0, 0, 0),
    "rotation": (0, 0, 0),
    "scale": (1, 1, 1),
    "shade": "flat",
    "modifiers": [
        {
            "type": "decimate",
            "ratio": 0.6
        },
        {
            "type": "triangulate"
        }
    ],
    "uv": {
        "mode": "smart_project",
        "angle_limit": 45.0
    },
    "export": {
        "tangents": False
    }
}
