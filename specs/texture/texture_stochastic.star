# Stochastic tiling texture spec using wang_tiles_node() and texture_bomb_node()
#
# Demonstrates seamless random tiling to reduce visible repetition patterns.

nodes = [
    # Base noise pattern
    noise_node("base_noise", "perlin", 0.1, 4),

    # Wang tiles for seamless random tiling
    wang_tiles_node("tiled", "base_noise", 4, 4, 0.1),

    # Another noise for scattering
    noise_node("scatter_src", "worley", 0.2, 2),

    # Texture bombing with various settings
    texture_bomb_node("scattered", "scatter_src", 0.6, 0.7, 1.3, 90.0, "max"),

    # Combine both effects
    multiply_node("combined", "tiled", "scattered"),

    # Color map the result
    color_ramp_node("colored", "combined", ["#1a1a2e", "#16213e", "#0f3460", "#e94560"]),
]

spec(
    asset_id = "stdlib-texture-stochastic-01",
    asset_type = "texture",
    seed = 42,
    outputs = [output("textures/stochastic.png", "png", source = "colored")],
    recipe = {
        "kind": "texture.procedural_v1",
        "params": texture_graph([128, 128], nodes),
    },
)
