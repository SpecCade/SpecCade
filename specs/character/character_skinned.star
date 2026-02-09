# Skinned character mesh - demonstrates armature-driven stdlib functions
#
# This example creates a skinned humanoid character using armature-driven
# procedural mesh generation, demonstrating body_part helpers and material
# configuration.
#
# Functions covered:
# - spec with skeletal_mesh.armature_driven_v1: Creates an armature-driven skeletal mesh
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

# Main skinned mesh spec using armature-driven generation
spec(
    asset_id = "stdlib-character-skinned-01",
    asset_type = "skeletal_mesh",
    seed = 42,
    description = "Stdlib skinned character example",
    outputs = [output("characters/skinned_character.glb", "glb")],
    recipe = {
        "kind": "skeletal_mesh.armature_driven_v1",
        "params": {
            "skeleton_preset": "humanoid_connected_v1",
            "bone_meshes": {
                "hips": {
                    "profile": "circle(8)",
                    "profile_radius": {"absolute": 0.12},
                    "material_index": 0,
                    "cap_start": True,
                    "cap_end": False
                },
                "spine": {
                    "profile": "circle(8)",
                    "profile_radius": {"absolute": 0.13},
                    "material_index": 0,
                    "cap_start": False,
                    "cap_end": False
                },
                "chest": {
                    "profile": "circle(8)",
                    "profile_radius": {"absolute": 0.14},
                    "material_index": 0,
                    "cap_start": False,
                    "cap_end": True
                },
                "neck": {
                    "profile": "circle(8)",
                    "profile_radius": {"absolute": 0.05},
                    "material_index": 0,
                    "cap_start": True,
                    "cap_end": False
                },
                "head": {
                    "profile": "circle(8)",
                    "profile_radius": {"absolute": 0.05},
                    "material_index": 1,
                    "cap_start": False,
                    "cap_end": True,
                    "attachments": [
                        {
                            "primitive": "sphere",
                            "dimensions": [0.55, 0.65, 0.55],
                            "offset": [0, 0, 0.4],
                            "material_index": 1
                        }
                    ]
                },
                "shoulder_l": {"profile": "circle(6)", "profile_radius": {"absolute": 0.06}, "cap_start": True, "cap_end": False, "material_index": 0},
                "shoulder_r": {"mirror": "shoulder_l"},
                "upper_arm_l": {
                    "profile": "circle(6)",
                    "profile_radius": {"absolute": 0.05},
                    "material_index": 0
                },
                "upper_arm_r": {"mirror": "upper_arm_l"},
                "lower_arm_l": {
                    "profile": "circle(6)",
                    "profile_radius": {"absolute": 0.04},
                    "material_index": 0,
                    "cap_end": True
                },
                "lower_arm_r": {"mirror": "lower_arm_l"},
                "hand_l": {
                    "profile": "circle(6)",
                    "profile_radius": {"absolute": 0.03},
                    "material_index": 0,
                    "cap_end": True
                },
                "hand_r": {"mirror": "hand_l"},
                "upper_leg_l": {
                    "profile": "circle(6)",
                    "profile_radius": {"absolute": 0.07},
                    "material_index": 0,
                    "cap_start": True
                },
                "upper_leg_r": {"mirror": "upper_leg_l"},
                "lower_leg_l": {
                    "profile": "circle(6)",
                    "profile_radius": {"absolute": 0.05},
                    "material_index": 0,
                    "cap_end": True
                },
                "lower_leg_r": {"mirror": "lower_leg_l"},
                "foot_l": {
                    "profile": "circle(6)",
                    "profile_radius": {"absolute": 0.04},
                    "material_index": 0,
                    "cap_end": True
                },
                "foot_r": {"mirror": "foot_l"}
            },
            "material_slots": [
                material_slot(
                    name = "body_material",
                    base_color = [0.8, 0.6, 0.5, 1.0],
                    roughness = 0.6
                ),
                material_slot(
                    name = "head_material",
                    base_color = [0.9, 0.7, 0.6, 1.0],
                    roughness = 0.5
                )
            ]
        }
    }
)
