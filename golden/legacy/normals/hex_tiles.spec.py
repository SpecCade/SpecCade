# Legacy .spec.py - REFERENCE ONLY (DO NOT EXECUTE)
# Hexagonal tile pattern normal map

NORMAL = {
    "name": "hex_tiles",
    "size": [256, 256],
    "method": "from_pattern",
    "pattern": {
        "type": "hexagons",
        "hex_size": 32,
        "gap_width": 3,
        "gap_depth": 0.25,
        "seed": 567
    },
    "processing": {
        "strength": 1.0,
        "blur": 0.3,
        "invert": True
    }
}
