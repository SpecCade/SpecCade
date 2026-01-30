# Humanoid male (golden fixture) - armature-driven using humanoid_basic_v1 preset

spec(
    asset_id = "humanoid_male",
    asset_type = "skeletal_mesh",
    seed = 7101,
    license = "CC0-1.0",
    description = "Golden fixture: humanoid male (armature-driven, preset skeleton)",
    tags = ["golden", "skeletal_mesh", "character", "humanoid", "male", "armature_driven_v1"],
    outputs = [output("skeletal_mesh/humanoid_male.glb", "glb")],
    recipe = {
        "kind": "skeletal_mesh.armature_driven_v1",
        "params": {
            "skeleton_preset": "humanoid_basic_v1",
            "material_slots": [
                {"name": "skin", "base_color": [0.82, 0.69, 0.59, 1.0], "roughness": 0.55},
                {"name": "cloth", "base_color": [0.12, 0.15, 0.18, 1.0], "roughness": 0.9},
                {"name": "metal", "base_color": [0.55, 0.56, 0.58, 1.0], "metallic": 1.0, "roughness": 0.25},
            ],
            "bool_shapes": {
                "jaw_cut": {
                    "primitive": "cube",
                    "dimensions": [0.22, 0.16, 0.10],
                    "position": [0.0, 0.30, -0.08],
                    "bone": "head",
                },
            },
            "bone_meshes": {
                "spine": {
                    "profile": "circle(12)",
                    "profile_radius": 0.115,
                    "taper": 0.95,
                    "bulge": [{"at": 0.30, "scale": 1.10}, {"at": 0.75, "scale": 0.92}],
                    "material_index": 1,
                },
                "chest": {
                    "profile": "hexagon(8)",
                    "profile_radius": [0.16, 0.12],
                    "taper": 0.88,
                    "translate": [0.0, 0.02, 0.0],
                    "rotate": [-3.0, 0.0, 0.0],
                    "bulge": [{"at": 0.25, "scale": 1.20}, {"at": 0.85, "scale": 0.85}],
                    "material_index": 1,
                    "modifiers": [{"bevel": {"width": 0.015, "segments": 1}}],
                    "attachments": [
                        {
                            "primitive": "cube",
                            "dimensions": [0.26, 0.20, 0.06],
                            "offset": [0.0, 0.25, 0.10],
                            "rotation": [0.0, 0.0, 10.0],
                            "material_index": 2,
                        },
                    ],
                },
                "head": {
                    "profile": "circle(16)",
                    "profile_radius": {"absolute": 0.12},
                    "taper": 1.0,
                    "bulge": [{"at": 0.10, "scale": 1.10}, {"at": 0.55, "scale": 1.25}],
                    "material_index": 0,
                    "modifiers": [{"bool": {"operation": "difference", "target": "jaw_cut"}}],
                    "attachments": [
                        {
                            "primitive": "ico_sphere",
                            "dimensions": [0.06, 0.06, 0.06],
                            "offset": [0.0, 0.36, 0.16],
                            "rotation": [0.0, 0.0, 0.0],
                            "material_index": 2,
                        },
                    ],
                },
                "upper_arm_l": {
                    "profile": "rectangle",
                    "profile_radius": [0.085, 0.12],
                    "taper": 0.80,
                    "rotate": [0.0, 0.0, -15.0],
                    "bulge": [{"at": 0.22, "scale": 1.15}, {"at": 0.75, "scale": 0.90}],
                    "material_index": 0,
                },
                "upper_arm_r": {"mirror": "upper_arm_l"},
                "upper_leg_l": {
                    "profile": "circle(10)",
                    "profile_radius": 0.12,
                    "taper": 0.78,
                    "bulge": [{"at": 0.30, "scale": 1.10}],
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
