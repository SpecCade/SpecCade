# Mixed armature-driven mode: extrusion body + modular part accessories.
#
# [VALIDATION]
# SHAPE: Organic torso (extrusion) with hard-surface shoulder/head armor parts.
# PROPORTIONS: Torso is rounded, accessory pieces are thicker and segmented.
# ORIENTATION: Upright humanoid with mirrored shoulder armor.
# FRONT VIEW: Rounded body core plus two shell-like shoulder plates.
# ISO VIEW: Clear coexistence of extrusion and part-composed meshes.

spec(
    asset_id = "part_mixed",
    asset_type = "skeletal_mesh",
    seed = 4103,
    license = "CC0-1.0",
    description = "Mixed extrusion and modular part workflow in armature_driven_v1",
    tags = ["skeletal_mesh", "character", "armature_driven_v1", "modular_parts", "mixed_mode"],
    outputs = [output("skeletal_mesh/part_mixed.glb", "glb")],
    recipe = {
        "kind": "skeletal_mesh.armature_driven_v1",
        "params": {
            "skeleton_preset": "humanoid_basic_v1",
            "material_slots": [
                material_slot(name = "organic", base_color = [0.65, 0.57, 0.48, 1.0], roughness = 0.62),
                material_slot(name = "armor", base_color = [0.35, 0.38, 0.44, 1.0], metallic = 0.65, roughness = 0.28),
            ],
            "bone_meshes": {
                "spine": {
                    "profile": "circle(10)",
                    "profile_radius": {"absolute": 0.12},
                    "extrusion_steps": [
                        {"extrude": 0.30, "scale": 1.12},
                        {"extrude": 0.45, "scale": 1.06},
                        {"extrude": 0.25, "scale": 0.82},
                    ],
                    "material_index": 0,
                },
                "chest": {
                    "profile": "circle(10)",
                    "profile_radius": {"absolute": 0.14},
                    "extrusion_steps": [
                        {"extrude": 0.20, "scale": 1.10},
                        {"extrude": 0.50, "scale": 1.05},
                        {"extrude": 0.30, "scale": 0.72},
                    ],
                    "material_index": 0,
                },
                "shoulder_l": {
                    "part": {
                        "base": {"primitive": "sphere", "dimensions": [0.18, 0.14, 0.12], "offset": [0.0, 0.0, 0.55]},
                        "operations": [
                            {"op": "difference", "target": {"primitive": "sphere", "dimensions": [0.14, 0.10, 0.10], "offset": [0.0, 0.0, 0.55]}},
                        ],
                        "scale": {"axes": []},
                    },
                    "material_index": 1,
                },
                "shoulder_r": {"mirror": "shoulder_l"},
                "head": {
                    "part": {
                        "base": {"primitive": "cube", "dimensions": [0.22, 0.20, 0.95]},
                        "operations": [
                            {"op": "difference", "target": {"primitive": "cube", "dimensions": [0.16, 0.06, 0.24], "offset": [0.0, 0.10, 0.52]}},
                            {"op": "union", "target": {"primitive": "cylinder", "dimensions": [0.016, 0.016, 0.32], "offset": [0.07, 0.0, 0.92]}},
                        ],
                    },
                    "material_index": 1,
                },
            },
        },
    },
)
