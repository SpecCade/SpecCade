# Legacy .spec.py - REFERENCE ONLY (DO NOT EXECUTE)
# Brick wall texture - tileable with layered noise

TEXTURE = {
    "name": "brick_wall",
    "size": [512, 512],
    "layers": [
        {
            "type": "brick",
            "brick_width": 64,
            "brick_height": 32,
            "mortar_width": 4,
            "mortar_color": 0.3,
            "brick_color": 0.7,
            "variation": 0.1,
            "seed": 42
        },
        {
            "type": "noise",
            "noise_type": "perlin",
            "scale": 0.05,
            "octaves": 3,
            "seed": 123,
            "blend": "multiply",
            "opacity": 0.3
        }
    ],
    "color_ramp": ["#8B4513", "#A0522D", "#CD853F"]
}
