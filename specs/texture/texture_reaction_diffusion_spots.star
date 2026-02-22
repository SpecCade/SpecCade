# Golden coverage: reaction_diffusion_preset enum value "spots".

rd = reaction_diffusion_preset("spots")

spec(
    asset_id = "stdlib-texture-reaction-diffusion-spots-01",
    asset_type = "texture",
    seed = 447,
    outputs = [output("textures/reaction_diffusion_spots.png", "png", source = "rd")],
    recipe = {
        "kind": "texture.procedural_v1",
        "params": texture_graph(
            [96, 96],
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
    description = "Reaction-diffusion grayscale texture using spots preset"
)
