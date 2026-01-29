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
    bone_meshes = {
        "spine": {
            "profile": "circle(8)",
            "profile_radius": 0.15,
            "material_index": 0,
            "cap_start": True,
            "cap_end": False,
        },
        "chest": {
            "profile": "circle(8)",
            "profile_radius": 0.18,
            "material_index": 0,
            "cap_start": False,
            "cap_end": True,
        },
        "head": {
            "profile": "circle(8)",
            "profile_radius": 0.08,
            "material_index": 0,
            "cap_start": False,
            "cap_end": True,
            "attachments": [
                {
                    "primitive": "sphere",
                    "dimensions": [0.15, 0.18, 0.15],
                    "offset": [0, 0, 0.6],
                    "material_index": 0,
                },
            ],
        },
        "upper_arm_l": {
            "profile": "circle(6)",
            "profile_radius": 0.08,
            "material_index": 0,
            "rotate": [0, 0, 90],
        },
        "upper_arm_r": {
            "profile": "circle(6)",
            "profile_radius": 0.08,
            "material_index": 0,
            "rotate": [0, 0, -90],
        },
        "upper_leg_l": {
            "profile": "circle(6)",
            "profile_radius": 0.1,
            "material_index": 0,
            "rotate": [180, 0, 0],
        },
        "upper_leg_r": {
            "profile": "circle(6)",
            "profile_radius": 0.1,
            "material_index": 0,
            "rotate": [180, 0, 0],
        },
    },
    material_slots = [
        material_slot(
            name = "blank_white",
            base_color = [1.0, 1.0, 1.0, 1.0]
        ),
    ],
    export = skeletal_export_settings(
        triangulate = True,
        include_skin_weights = True
    ),
    constraints = skeletal_constraints(
        max_triangles = 5000,
        max_bones = 64,
        max_materials = 4
    )
)
