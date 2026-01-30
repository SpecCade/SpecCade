# Legacy .spec.py - REFERENCE ONLY (DO NOT EXECUTE)
# Checkerboard floor texture - non-tileable center focus

TEXTURE = {
    "name": "checker_floor",
    "size": [256, 256],
    "layers": [
        {
            "type": "checkerboard",
            "tile_size": 32,
            "color1": 0.2,
            "color2": 0.8
        },
        {
            "type": "noise",
            "noise_type": "perlin",
            "scale": 0.08,
            "octaves": 2,
            "seed": 555,
            "blend": "multiply",
            "opacity": 0.1
        }
    ],
    "palette": ["#1a1a1a", "#e0e0e0"]
}
