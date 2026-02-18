# Decal golden spec
#
# Exercises the texture.decal_v1 recipe with procedural graph nodes for albedo,
# alpha, normal, and roughness outputs, plus full DecalMetadata with projection
# settings.

decal_spec(
    asset_id = "golden-texture-decal-01",
    seed = 2002,
    output_path = "textures/decal_scorch_mark.png",
    resolution = [256, 256],
    nodes = [
        noise_node("base_noise", "worley", 12.0, 3, 0.6, 2.0),
        noise_node("detail", "perlin", 24.0, 2),
        multiply_node("combined", "base_noise", "detail"),
        threshold_node("alpha_mask", "combined", 0.35),
        invert_node("inv_base", "base_noise"),
        normal_from_height_node("normals", "inv_base", 1.5),
        clamp_node("roughness", "combined", 0.2, 0.9),
    ],
    albedo_output = "combined",
    alpha_output = "alpha_mask",
    normal_output = "normals",
    roughness_output = "roughness",
    metadata = decal_metadata(
        aspect_ratio = 1.0,
        anchor = [0.5, 0.5],
        fade_distance = 0.15,
        projection_size = [0.8, 0.8],
        depth_range = [0.0, 0.05],
    ),
    description = "Scorch-mark decal: tests all decal outputs (albedo, alpha, normal, roughness) and full projection metadata",
    tags = ["golden", "decal", "damage"]
)
