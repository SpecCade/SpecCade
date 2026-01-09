# Legacy .spec.py - REFERENCE ONLY (DO NOT EXECUTE)
# Torus ring - donut shape

MESH = {
    "name": "torus_ring",
    "primitive": "torus",
    "params": {
        "major_radius": 0.5,
        "minor_radius": 0.15,
        "major_segments": 48,
        "minor_segments": 24
    },
    "location": (0, 0, 0),
    "rotation": (1.5708, 0, 0),
    "scale": (1, 1, 1),
    "shade": "smooth",
    "modifiers": [],
    "uv": {
        "mode": "smart_project",
        "angle_limit": 60.0
    },
    "export": {
        "tangents": True
    }
}
