# Normal map from height example
#
# Generates a normal map from a heightmap for 3D lighting.
# Converts grayscale height data into RGB surface normals.
# Covers: normal_from_height_node()

spec(
    asset_id = "stdlib-texture-normalmap-01",
    asset_type = "texture",
    seed = 42,
    outputs = [output("textures/normal.png", "png", source = "normals")],
    recipe = {
        "kind": "texture.procedural_v1",
        "params": texture_graph(
            [256, 256],
            [
                # Create height field from noise
                noise_node("height", "perlin", 0.05, 6, 0.5, 2.0),

                # Optional: add some detail noise
                noise_node("detail", "simplex", 0.2, 2, 0.3, 2.5),
                constant_node("scale", 0.3),
                multiply_node("detail_scaled", "detail", "scale"),
                add_node("combined_height", "height", "detail_scaled"),

                # Clamp to valid range
                clamp_node("clamped", "combined_height", 0.0, 1.0),

                # Generate normal map with moderate strength
                normal_from_height_node("normals", "clamped", 1.5)
            ],
            True
        )
    }
)
