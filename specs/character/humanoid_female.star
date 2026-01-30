# Humanoid female (golden fixture) - armature-driven using humanoid_basic_v1 preset

spec(
    asset_id = "humanoid_female",
    asset_type = "skeletal_mesh",
    seed = 7102,
    license = "CC0-1.0",
    description = "Golden fixture: humanoid female (armature-driven, preset skeleton)",
    tags = ["golden", "skeletal_mesh", "character", "humanoid", "female", "armature_driven_v1"],
    outputs = [output("skeletal_mesh/humanoid_female.glb", "glb")],
    recipe = {
        "kind": "skeletal_mesh.armature_driven_v1",
        "params": {
            "skeleton_preset": "humanoid_basic_v1",
            "material_slots": [
                {"name": "skin", "base_color": [0.86, 0.73, 0.64, 1.0], "roughness": 0.55},
                {"name": "cloth", "base_color": [0.20, 0.18, 0.16, 1.0], "roughness": 0.85},
                {"name": "accent", "base_color": [0.60, 0.20, 0.18, 1.0], "roughness": 0.6},
            ],
            "bool_shapes": {
                "eye_cut_l": {
                    "primitive": "sphere",
                    "dimensions": [0.07, 0.07, 0.07],
                    "position": [-0.18, 0.22, 0.14],
                    "bone": "head",
                },
                "eye_cut_r": {"mirror": "eye_cut_l"},
            },
            "bone_meshes": {
                "spine": {
                    "profile": "circle(12)",
                    "profile_radius": 0.105,
                    "taper": 0.96,
                    "bulge": [{"at": 0.35, "scale": 1.08}],
                    "material_index": 1,
                },
                "chest": {
                    "profile": "hexagon(8)",
                    "profile_radius": [0.14, 0.11],
                    "taper": 0.90,
                    "translate": [0.0, 0.04, 0.0],
                    "rotate": [-2.0, 0.0, 0.0],
                    "bulge": [{"at": 0.30, "scale": 1.18}, {"at": 0.86, "scale": 0.88}],
                    "material_index": 1,
                    "attachments": [
                        {
                            "primitive": "torus",
                            "dimensions": [0.18, 0.18, 0.05],
                            "offset": [0.0, 0.20, 0.35],
                            "rotation": [90.0, 0.0, 0.0],
                            "material_index": 2,
                        },
                    ],
                },
                "head": {
                    "profile": "circle(16)",
                    "profile_radius": {"absolute": 0.115},
                    "taper": 1.0,
                    "bulge": [{"at": 0.10, "scale": 1.08}, {"at": 0.55, "scale": 1.22}],
                    "material_index": 0,
                    "modifiers": [
                        {"bool": {"operation": "difference", "target": "eye_cut_l"}},
                        {"bool": {"operation": "difference", "target": "eye_cut_r"}},
                    ],
                    "attachments": [
                        {
                            "extrude": {
                                "profile": "rectangle",
                                "start": [0.0, -0.02, 0.15],
                                "end": [0.0, -0.25, 0.10],
                                "profile_radius": [0.06, 0.10],
                                "taper": 0.7,
                            },
                        },
                    ],
                },
                "upper_arm_l": {
                    "profile": "rectangle",
                    "profile_radius": [0.075, 0.105],
                    "taper": 0.82,
                    "rotate": [0.0, 0.0, -12.0],
                    "bulge": [{"at": 0.18, "scale": 1.12}, {"at": 0.70, "scale": 0.92}],
                    "material_index": 0,
                },
                "upper_arm_r": {"mirror": "upper_arm_l"},
                "upper_leg_l": {
                    "profile": "circle(10)",
                    "profile_radius": 0.125,
                    "taper": 0.75,
                    "bulge": [{"at": 0.20, "scale": 1.12}, {"at": 0.65, "scale": 0.95}],
                    "material_index": 0,
                },
                "upper_leg_r": {"mirror": "upper_leg_l"},
            },
            "export": {
                "include_armature": True,
                "include_normals": True,
                "include_uvs": True,
                "triangulate": True,
                "include_skin_weights": True,
                "save_blend": False,
            },
            "constraints": {"max_triangles": 12000, "max_bones": 64, "max_materials": 8},
        },
    },
)
