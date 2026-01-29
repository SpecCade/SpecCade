# Skinned character mesh - demonstrates skinned mesh stdlib functions
#
# This example creates a skinned humanoid character that binds an external
# mesh file to a skeleton, using body_part, skeletal_texturing, and
# skinning_config helpers.
#
# Functions covered:
# - skeletal_mesh_skinned_spec: Creates a spec that binds an existing mesh to a skeleton
# - body_part: Defines body part geometry attached to bones
# - skeletal_texturing: Configures UV unwrapping for the mesh
# - skinning_config: Configures bone influence weights

# Example body parts for reference (demonstrates body_part function)
TORSO = body_part(
    bone = "spine",
    primitive = "cylinder",
    dimensions = [0.25, 0.25, 0.35],
    segments = 8,
    offset = [0, 0, 0.1],
    material_index = 0
)

HEAD = body_part(
    bone = "head",
    primitive = "sphere",
    dimensions = [0.15, 0.18, 0.15],
    segments = 12,
    offset = [0, 0, 0.08],
    rotation = [0, 0, 0],
    material_index = 1
)

ARM_LEFT = body_part(
    bone = "upper_arm_l",
    primitive = "cylinder",
    dimensions = [0.06, 0.06, 0.25],
    segments = 6,
    material_index = 0
)

LEG_LEFT = body_part(
    bone = "upper_leg_l",
    primitive = "cylinder",
    dimensions = [0.08, 0.08, 0.35],
    segments = 6,
    offset = [0, 0, -0.02],
    material_index = 0
)

# Texturing configuration (demonstrates skeletal_texturing function)
TEXTURING_CONFIG = skeletal_texturing(uv_mode = "cylinder_project")
TEXTURING_SMART = skeletal_texturing(uv_mode = "smart")
TEXTURING_BOX = skeletal_texturing(uv_mode = "box_project")

# Skinning configurations (demonstrates skinning_config function)
SKINNING_DEFAULT = skinning_config()
SKINNING_MOBILE = skinning_config(max_bone_influences = 2, auto_weights = True)
SKINNING_HIGH_QUALITY = skinning_config(max_bone_influences = 8, auto_weights = True)
SKINNING_MANUAL = skinning_config(max_bone_influences = 4, auto_weights = False)

# Main skinned mesh spec (demonstrates skeletal_mesh_skinned_spec function)
skeletal_mesh_skinned_spec(
    asset_id = "stdlib-character-skinned-01",
    seed = 42,
    output_path = "characters/skinned_character.glb",
    format = "glb",
    mesh_file = "assets/humanoid_mesh.glb",
    skeleton_preset = "humanoid_basic_v1",
    binding = {
        "mode": "auto_weights",
        "max_bone_influences": 4,
        "vertex_group_map": {}
    },
    material_slots = [
        material_slot(
            name = "body_material",
            base_color = [0.8, 0.6, 0.5, 1.0],
            roughness = 0.6
        ),
        material_slot(
            name = "head_material",
            base_color = [0.9, 0.7, 0.6, 1.0],
            roughness = 0.5
        ),
    ],
    export = skeletal_export_settings(
        triangulate = True,
        include_skin_weights = True
    ),
    constraints = skeletal_constraints(
        max_triangles = 8000,
        max_bones = 64,
        max_materials = 8
    ),
    description = "Stdlib skinned character example"
)
