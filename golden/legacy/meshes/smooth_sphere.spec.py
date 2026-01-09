# Legacy .spec.py - REFERENCE ONLY (DO NOT EXECUTE)
# High-poly UV sphere - smooth shading

MESH = {
    "name": "smooth_sphere",
    "primitive": "sphere",
    "params": {
        "radius": 0.5,
        "segments": 48,
        "rings": 24
    },
    "location": (0, 0, 0),
    "rotation": (0, 0, 0),
    "scale": (1, 1, 1),
    "shade": "smooth",
    "modifiers": [],
    "uv": {
        "mode": "smart_project",
        "angle_limit": 66.0
    },
    "export": {
        "tangents": True
    }
}
