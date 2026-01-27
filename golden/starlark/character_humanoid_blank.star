# Blank-material humanoid character - placeholder for texturing
#
# All body parts use a single white material (material_index=0).
# Use this as a starting template when the final look will come
# from textures or runtime shading rather than baked vertex colours.

skeletal_mesh_spec(
    asset_id = "stdlib-character-humanoid-blank-01",
    seed = 42,
    output_path = "characters/humanoid_blank.glb",
    format = "glb",
    skeleton_preset = "humanoid_basic_v1",
    description = "Blank-material humanoid placeholder",
    body_parts = [
        # Torso
        body_part(
            bone = "spine",
            primitive = "cylinder",
            dimensions = [0.25, 0.4, 0.25],
            segments = 8,
            offset = [0, 0, 0.3],
            material_index = 0
        ),
        body_part(
            bone = "chest",
            primitive = "cylinder",
            dimensions = [0.3, 0.3, 0.28],
            segments = 8,
            offset = [0, 0, 0.6],
            material_index = 0
        ),
        # Head
        body_part(
            bone = "head",
            primitive = "sphere",
            dimensions = [0.15, 0.18, 0.15],
            segments = 12,
            offset = [0, 0, 0.95],
            material_index = 0
        ),
        # Arms
        body_part(
            bone = "upper_arm_l",
            primitive = "cylinder",
            dimensions = [0.06, 0.25, 0.06],
            segments = 6,
            rotation = [0, 0, 90],
            material_index = 0
        ),
        body_part(
            bone = "upper_arm_r",
            primitive = "cylinder",
            dimensions = [0.06, 0.25, 0.06],
            segments = 6,
            rotation = [0, 0, -90],
            material_index = 0
        ),
        # Legs
        body_part(
            bone = "upper_leg_l",
            primitive = "cylinder",
            dimensions = [0.08, 0.35, 0.08],
            segments = 6,
            rotation = [180, 0, 0],
            material_index = 0
        ),
        body_part(
            bone = "upper_leg_r",
            primitive = "cylinder",
            dimensions = [0.08, 0.35, 0.08],
            segments = 6,
            rotation = [180, 0, 0],
            material_index = 0
        ),
    ],
    material_slots = [
        material_slot(
            name = "blank_white",
            base_color = [1.0, 1.0, 1.0, 1.0]
        ),
    ],
    skinning = skinning_config(
        max_bone_influences = 4,
        auto_weights = True
    ),
    export = skeletal_export_settings(
        triangulate = True,
        include_skin_weights = True
    ),
    constraints = skeletal_constraints(
        max_triangles = 5000,
        max_bones = 64,
        max_materials = 4
    ),
    texturing = skeletal_texturing(uv_mode = "cylinder_project")
)
