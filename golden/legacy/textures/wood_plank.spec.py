# Legacy .spec.py - REFERENCE ONLY (DO NOT EXECUTE)
# Wood plank texture - grain pattern with variation

TEXTURE = {
    "name": "wood_plank",
    "size": [512, 512],
    "layers": [
        {
            "type": "wood_grain",
            "ring_count": 12,
            "distortion": 0.25,
            "seed": 789
        },
        {
            "type": "noise",
            "noise_type": "perlin",
            "scale": 0.02,
            "octaves": 4,
            "seed": 321,
            "blend": "multiply",
            "opacity": 0.2
        },
        {
            "type": "stripes",
            "direction": "vertical",
            "stripe_width": 128,
            "color1": 0.4,
            "color2": 0.5,
            "blend": "soft_light",
            "opacity": 0.1
        }
    ],
    "color_ramp": ["#8B4513", "#A0522D", "#DEB887", "#D2691E"]
}
