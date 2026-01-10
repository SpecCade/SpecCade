# Legacy .spec.py - REFERENCE ONLY (DO NOT EXECUTE)
# Brick pattern normal map - matching brick_wall texture

NORMAL = {
    "name": "brick_normal",
    "size": [512, 512],
    "method": "from_pattern",
    "pattern": {
        "type": "bricks",
        "brick_width": 64,
        "brick_height": 32,
        "mortar_width": 4,
        "mortar_depth": 0.4,
        "brick_variation": 0.08,
        "seed": 42
    },
    "processing": {
        "strength": 1.2,
        "blur": 0.5,
        "invert": True
    }
}
