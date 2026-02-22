# Golden coverage: noise_node algorithm enum value "gabor".

spec(
    asset_id = "stdlib-texture-gabor-noise-01",
    asset_type = "texture",
    seed = 444,
    outputs = [output("textures/gabor_noise.png", "png", source = "gabor_noise")],
    recipe = {
        "kind": "texture.procedural_v1",
        "params": texture_graph(
            [96, 96],
            [
                noise_node("gabor_noise", "gabor", 0.12, 1, 0.5, 2.0),
            ],
            True
        )
    },
    description = "Gabor noise texture - enum coverage for algorithm::gabor"
)
