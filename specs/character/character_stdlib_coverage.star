# Character stdlib coverage spec
#
# Covers all character stdlib functions:
# - skeletal_mesh_spec()
# - skeletal_mesh_skinned_spec()
# - custom_bone()
# - skeletal_export_settings()
# - skeletal_constraints()

# Example usage of skeletal_mesh_skinned_spec() (for coverage)
skinned_spec = skeletal_mesh_skinned_spec(
    asset_id = "stdlib-character-skinned-coverage-01",
    seed = 100,
    output_path = "characters/skinned_coverage.glb",
    format = "glb",
    mesh_file = "meshes/base_character.glb",
    skeleton_preset = "humanoid_basic_v1",
    binding = {
        "mode": "auto_weights",
        "vertex_group_map": {},
        "max_bone_influences": 4
    },
    description = "Coverage test for skeletal_mesh_skinned_spec"
)

# Example usage of skeletal_mesh_spec() with helper functions
skeletal_mesh_spec(
    asset_id = "stdlib-character-coverage-01",
    seed = 42,
    output_path = "characters/coverage.glb",
    format = "glb",
    skeleton = [
        custom_bone(
            bone = "root",
            head = [0.0, 0.0, 0.0],
            tail = [0.0, 0.0, 0.1]
        ),
        custom_bone(
            bone = "spine",
            parent = "root",
            head = [0.0, 0.0, 0.1],
            tail = [0.0, 0.0, 0.3]
        ),
        custom_bone(
            bone = "chest",
            parent = "spine",
            head = [0.0, 0.0, 0.3],
            tail = [0.0, 0.0, 0.5]
        )
    ],
    bone_meshes = {
        "spine": {
            "profile": "circle(8)",
            "profile_radius": 0.12,
            "cap_start": True,
            "cap_end": False
        },
        "chest": {
            "profile": "circle(8)",
            "profile_radius": 0.15,
            "cap_start": False,
            "cap_end": True
        }
    },
    material_slots = [
        material_slot(
            name = "body_material",
            base_color = [0.7, 0.5, 0.4, 1.0],
            metallic = 0.0,
            roughness = 0.8
        )
    ],
    export = skeletal_export_settings(
        include_armature = True,
        include_normals = True,
        include_uvs = True,
        triangulate = True,
        include_skin_weights = True,
        save_blend = False
    ),
    constraints = skeletal_constraints(
        max_triangles = 5000,
        max_bones = 64,
        max_materials = 4
    ),
    description = "Coverage test for character stdlib functions"
)
