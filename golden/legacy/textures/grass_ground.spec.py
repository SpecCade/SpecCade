# Legacy .spec.py - REFERENCE ONLY (DO NOT EXECUTE)
# Grass ground texture - organic noise-based tileable

TEXTURE = {
    "name": "grass_ground",
    "size": [512, 512],
    "layers": [
        {
            "type": "noise",
            "noise_type": "simplex",
            "scale": 0.03,
            "octaves": 5,
            "seed": 999
        },
        {
            "type": "noise",
            "noise_type": "voronoi",
            "scale": 0.1,
            "jitter": 0.8,
            "seed": 888,
            "blend": "overlay",
            "opacity": 0.3
        },
        {
            "type": "noise",
            "noise_type": "perlin",
            "scale": 0.01,
            "octaves": 2,
            "seed": 777,
            "blend": "multiply",
            "opacity": 0.2
        }
    ],
    "color_ramp": ["#228B22", "#32CD32", "#7CFC00", "#006400"]
}
