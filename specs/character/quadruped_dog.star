# Quadruped dog (golden fixture) - custom skeleton + armature-driven modeling

spec(
    asset_id = "quadruped_dog",
    asset_type = "skeletal_mesh",
    seed = 7110,
    license = "CC0-1.0",
    description = "Golden fixture: quadruped dog (custom skeleton, armature-driven)",
    tags = ["golden", "skeletal_mesh", "character", "quadruped", "dog", "armature_driven_v1"],
    outputs = [output("skeletal_mesh/quadruped_dog.glb", "glb")],
    recipe = {
        "kind": "skeletal_mesh.armature_driven_v1",
        "params": {
            "skeleton": [
                {
                    "bone": "root",
                    "head": [0.0, 0.0, 0.0],
                    "tail": [0.0, 0.0, 0.12],
                },
                {
                    "bone": "spine",
                    "head": [0.0, 0.0, 0.12],
                    "tail": [0.0, 0.0, 0.45],
                    "parent": "root",
                },
                {
                    "bone": "chest",
                    "head": [0.0, 0.0, 0.45],
                    "tail": [0.0, 0.0, 0.60],
                    "parent": "spine",
                },
                {
                    "bone": "neck",
                    "head": [0.0, 0.0, 0.60],
                    "tail": [0.0, 0.30, 0.66],
                    "parent": "chest",
                },
                {
                    "bone": "head",
                    "head": [0.0, 0.30, 0.66],
                    "tail": [0.0, 0.55, 0.66],
                    "parent": "neck",
                },
                {
                    "bone": "tail_base",
                    "head": [0.0, -0.10, 0.44],
                    "tail": [0.0, -0.35, 0.50],
                    "parent": "spine",
                },
                {
                    "bone": "tail_tip",
                    "head": [0.0, -0.35, 0.50],
                    "tail": [0.0, -0.60, 0.56],
                    "parent": "tail_base",
                },
                {
                    "bone": "front_upper_leg_l",
                    "head": [-0.16, 0.35, 0.46],
                    "tail": [-0.16, 0.35, 0.18],
                    "parent": "chest",
                },
                {
                    "bone": "front_lower_leg_l",
                    "head": [-0.16, 0.35, 0.18],
                    "tail": [-0.16, 0.35, 0.02],
                    "parent": "front_upper_leg_l",
                },
                {
                    "bone": "front_paw_l",
                    "head": [-0.16, 0.35, 0.02],
                    "tail": [-0.16, 0.52, 0.02],
                    "parent": "front_lower_leg_l",
                },
                {
                    "bone": "front_upper_leg_r",
                    "mirror": "front_upper_leg_l",
                },
                {
                    "bone": "front_lower_leg_r",
                    "mirror": "front_lower_leg_l",
                },
                {
                    "bone": "front_paw_r",
                    "mirror": "front_paw_l",
                },
                {
                    "bone": "hind_upper_leg_l",
                    "head": [-0.18, -0.08, 0.42],
                    "tail": [-0.18, -0.08, 0.16],
                    "parent": "spine",
                },
                {
                    "bone": "hind_lower_leg_l",
                    "head": [-0.18, -0.08, 0.16],
                    "tail": [-0.18, -0.08, 0.02],
                    "parent": "hind_upper_leg_l",
                },
                {
                    "bone": "hind_paw_l",
                    "head": [-0.18, -0.08, 0.02],
                    "tail": [-0.18, 0.06, 0.02],
                    "parent": "hind_lower_leg_l",
                },
                {
                    "bone": "hind_upper_leg_r",
                    "mirror": "hind_upper_leg_l",
                },
                {
                    "bone": "hind_lower_leg_r",
                    "mirror": "hind_lower_leg_l",
                },
                {
                    "bone": "hind_paw_r",
                    "mirror": "hind_paw_l",
                },
            ],
            "bone_meshes": {
                "root": {
                    "profile": "circle(10)",
                    "profile_radius": 0.10,
                    "material_index": 0,
                },
                "spine": {
                    "profile": "circle(12)",
                    "profile_radius": 0.11,
                    "taper": 0.88,
                    "bulge": [
                        {"at": 0.30, "scale": 1.15},
                        {"at": 0.85, "scale": 0.82},
                    ],
                    "material_index": 0,
                },
                "chest": {
                    "profile": "hexagon(8)",
                    "profile_radius": [0.15, 0.12],
                    "taper": 0.90,
                    "material_index": 0,
                },
                "neck": {
                    "profile": "circle(10)",
                    "profile_radius": 0.08,
                    "taper": 0.85,
                    "material_index": 0,
                },
                "head": {
                    "profile": "circle(16)",
                    "profile_radius": {"absolute": 0.10},
                    "material_index": 0,
                    "attachments": [
                        {
                            "primitive": "cone",
                            "dimensions": [0.08, 0.08, 0.12],
                            "offset": [-0.08, 0.16, 0.10],
                            "rotation": [0.0, 0.0, -20.0],
                            "material_index": 1,
                        },
                        {
                            "primitive": "cone",
                            "dimensions": [0.08, 0.08, 0.12],
                            "offset": [0.08, 0.16, 0.10],
                            "rotation": [0.0, 0.0, 20.0],
                            "material_index": 1,
                        },
                    ],
                },
                "tail_base": {
                    "profile": "circle(8)",
                    "profile_radius": 0.06,
                    "taper": 0.70,
                    "material_index": 0,
                },
                "tail_tip": {
                    "profile": "circle(8)",
                    "profile_radius": 0.05,
                    "taper": 0.60,
                    "material_index": 0,
                },
                "front_upper_leg_l": {
                    "profile": "circle(8)",
                    "profile_radius": 0.06,
                    "taper": 0.80,
                    "material_index": 0,
                },
                "front_lower_leg_l": {
                    "profile": "circle(8)",
                    "profile_radius": 0.05,
                    "taper": 0.75,
                    "material_index": 0,
                },
                "front_paw_l": {
                    "profile": "circle(8)",
                    "profile_radius": 0.045,
                    "taper": 0.95,
                    "material_index": 0,
                },
                "front_upper_leg_r": {
                    "mirror": "front_upper_leg_l",
                },
                "front_lower_leg_r": {
                    "mirror": "front_lower_leg_l",
                },
                "front_paw_r": {
                    "mirror": "front_paw_l",
                },
                "hind_upper_leg_l": {
                    "profile": "circle(8)",
                    "profile_radius": 0.07,
                    "taper": 0.78,
                    "material_index": 0,
                },
                "hind_lower_leg_l": {
                    "profile": "circle(8)",
                    "profile_radius": 0.055,
                    "taper": 0.72,
                    "material_index": 0,
                },
                "hind_paw_l": {
                    "profile": "circle(8)",
                    "profile_radius": 0.05,
                    "taper": 0.95,
                    "material_index": 0,
                },
                "hind_upper_leg_r": {
                    "mirror": "hind_upper_leg_l",
                },
                "hind_lower_leg_r": {
                    "mirror": "hind_lower_leg_l",
                },
                "hind_paw_r": {
                    "mirror": "hind_paw_l",
                },
            },
            "material_slots": [
                {
                    "name": "fur",
                    "base_color": [0.45, 0.33, 0.22, 1.0],
                    "roughness": 0.9,
                },
                {
                    "name": "ear_tip",
                    "base_color": [0.12, 0.08, 0.06, 1.0],
                    "roughness": 0.9,
                },
            ],
            "export": {
                "include_armature": True,
                "include_normals": True,
                "include_uvs": True,
                "triangulate": True,
                "include_skin_weights": True,
                "save_blend": False,
            },
            "constraints": {
                "max_triangles": 9000,
                "max_bones": 64,
                "max_materials": 8,
            },
        },
    },
)
