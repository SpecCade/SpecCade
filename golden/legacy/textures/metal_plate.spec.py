# Legacy .spec.py - REFERENCE ONLY (DO NOT EXECUTE)
# Metal plate texture - tileable with scratches

TEXTURE = {
    "name": "metal_plate",
    "size": [256, 256],
    "layers": [
        {
            "type": "solid",
            "color": 0.5
        },
        {
            "type": "noise",
            "noise_type": "perlin",
            "scale": 0.1,
            "octaves": 2,
            "seed": 456,
            "blend": "add",
            "opacity": 0.15
        },
        {
            "type": "gradient",
            "direction": "radial",
            "center": [0.5, 0.5],
            "inner": 0.6,
            "outer": 0.4,
            "blend": "overlay",
            "opacity": 0.2
        }
    ],
    "color_ramp": ["#404040", "#606060", "#808080"]
}
