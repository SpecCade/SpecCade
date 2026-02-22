# Golden coverage: reaction_diffusion_node + reaction_diffusion_preset.

rd = reaction_diffusion_preset("mitosis")

spec(
    asset_id = "stdlib-texture-reaction-diffusion-01",
    asset_type = "texture",
    seed = 445,
    outputs = [output("textures/reaction_diffusion.png", "png", source = "rd")],
    recipe = {
        "kind": "texture.procedural_v1",
        "params": texture_graph(
            [128, 128],
            [
                reaction_diffusion_node(
                    "rd",
                    rd["steps"],
                    rd["feed"],
                    rd["kill"],
                    rd["diffuse_a"],
                    rd["diffuse_b"],
                    rd["dt"],
                    rd["seed_density"]
                ),
            ],
            True
        )
    },
    description = "Reaction-diffusion grayscale texture using preset parameters"
)
