# Legacy .spec.py - REFERENCE ONLY (DO NOT EXECUTE)
# Scratched metal surface normal map

NORMAL = {
    "name": "scratched_metal",
    "size": [256, 256],
    "method": "from_pattern",
    "pattern": {
        "type": "scratches",
        "density": 80,
        "length_range": [15, 60],
        "depth": 0.12,
        "seed": 234
    },
    "processing": {
        "strength": 0.8,
        "blur": 0.2,
        "invert": False
    }
}
