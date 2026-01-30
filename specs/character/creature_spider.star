# Creature spider (golden fixture) - custom skeleton + armature-driven modeling

spec(
    asset_id = "creature_spider",
    asset_type = "skeletal_mesh",
    seed = 7111,
    license = "CC0-1.0",
    description = "Golden fixture: creature spider (custom skeleton, mirrored legs)",
    tags = ["golden", "skeletal_mesh", "character", "creature", "spider", "armature_driven_v1"],
    outputs = [output("skeletal_mesh/creature_spider.glb", "glb")],
    recipe = {
        "kind": "skeletal_mesh.armature_driven_v1",
        "params": {
            "skeleton": [
                {
                    "bone": "root",
                    "head": [0.0, 0.0, 0.0],
                    "tail": [0.0, 0.0, 0.08],
                },
                {
                    "bone": "abdomen",
                    "head": [0.0, 0.0, 0.08],
                    "tail": [0.0, -0.20, 0.18],
                    "parent": "root",
                },
                {
                    "bone": "thorax",
                    "head": [0.0, 0.0, 0.08],
                    "tail": [0.0, 0.18, 0.16],
                    "parent": "root",
                },
                {
                    "bone": "head",
                    "head": [0.0, 0.18, 0.16],
                    "tail": [0.0, 0.32, 0.16],
                    "parent": "thorax",
                },
                {
                    "bone": "leg1_l",
                    "head": [-0.06, 0.10, 0.12],
                    "tail": [-0.28, 0.24, 0.05],
                    "parent": "thorax",
                },
                {
                    "bone": "leg2_l",
                    "head": [-0.06, 0.04, 0.11],
                    "tail": [-0.30, 0.10, 0.05],
                    "parent": "thorax",
                },
                {
                    "bone": "leg3_l",
                    "head": [-0.06, -0.02, 0.11],
                    "tail": [-0.30, -0.02, 0.05],
                    "parent": "thorax",
                },
                {
                    "bone": "leg4_l",
                    "head": [-0.06, -0.08, 0.12],
                    "tail": [-0.28, -0.18, 0.05],
                    "parent": "abdomen",
                },
                {
                    "bone": "leg1_r",
                    "mirror": "leg1_l",
                },
                {
                    "bone": "leg2_r",
                    "mirror": "leg2_l",
                },
                {
                    "bone": "leg3_r",
                    "mirror": "leg3_l",
                },
                {
                    "bone": "leg4_r",
                    "mirror": "leg4_l",
                },
            ],
            "bone_meshes": {
                "root": {
                    "profile": "circle(8)",
                    "profile_radius": 0.08,
                    "extrusion_steps": [
                        {"extrude": 1.0, "scale": 1.0},
                    ],
                    "material_index": 0,
                },
                "abdomen": {
                    "profile": "circle(16)",
                    "profile_radius": [0.16, 0.12],
                    "extrusion_steps": [
                        # 0% -> 25%: big bulge to 1.25
                        {"extrude": 0.25, "scale": 1.25},
                        # 25% -> 75%: taper to 1.05
                        {"extrude": 0.50, "scale": 1.05},
                        # 75% -> 100%: final taper to 0.70
                        {"extrude": 0.25, "scale": 0.70},
                    ],
                    "material_index": 0,
                },
                "thorax": {
                    "profile": "circle(16)",
                    "profile_radius": [0.13, 0.10],
                    "extrusion_steps": [
                        # 0% -> 30%: bulge to 1.15
                        {"extrude": 0.30, "scale": 1.15},
                        # 30% -> 100%: taper to 0.85
                        {"extrude": 0.70, "scale": 0.85},
                    ],
                    "material_index": 0,
                },
                "head": {
                    "profile": "circle(12)",
                    "profile_radius": {"absolute": 0.07},
                    "extrusion_steps": [
                        {"extrude": 1.0, "scale": 1.0},
                    ],
                    "material_index": 0,
                    "attachments": [
                        {
                            "primitive": "ico_sphere",
                            "dimensions": [0.03, 0.03, 0.03],
                            "offset": [-0.06, 0.20, 0.10],
                            "rotation": [0.0, 0.0, 0.0],
                            "material_index": 1,
                        },
                        {
                            "primitive": "ico_sphere",
                            "dimensions": [0.03, 0.03, 0.03],
                            "offset": [0.06, 0.20, 0.10],
                            "rotation": [0.0, 0.0, 0.0],
                            "material_index": 1,
                        },
                    ],
                },
                "leg1_l": {
                    "profile": "circle(8)",
                    "profile_radius": 0.04,
                    "extrusion_steps": [
                        {"extrude": 0.50, "scale": 0.75},
                        {"extrude": 0.50, "scale": 0.55},
                    ],
                    "material_index": 0,
                },
                "leg2_l": {
                    "profile": "circle(8)",
                    "profile_radius": 0.04,
                    "extrusion_steps": [
                        {"extrude": 0.50, "scale": 0.75},
                        {"extrude": 0.50, "scale": 0.55},
                    ],
                    "material_index": 0,
                },
                "leg3_l": {
                    "profile": "circle(8)",
                    "profile_radius": 0.04,
                    "extrusion_steps": [
                        {"extrude": 0.50, "scale": 0.75},
                        {"extrude": 0.50, "scale": 0.55},
                    ],
                    "material_index": 0,
                },
                "leg4_l": {
                    "profile": "circle(8)",
                    "profile_radius": 0.045,
                    "extrusion_steps": [
                        {"extrude": 0.50, "scale": 0.75},
                        {"extrude": 0.50, "scale": 0.55},
                    ],
                    "material_index": 0,
                },
                "leg1_r": {
                    "mirror": "leg1_l",
                },
                "leg2_r": {
                    "mirror": "leg2_l",
                },
                "leg3_r": {
                    "mirror": "leg3_l",
                },
                "leg4_r": {
                    "mirror": "leg4_l",
                },
            },
            "material_slots": [
                {
                    "name": "chitin",
                    "base_color": [0.08, 0.07, 0.06, 1.0],
                    "roughness": 0.85,
                },
                {
                    "name": "eyes",
                    "base_color": [0.75, 0.12, 0.10, 1.0],
                    "roughness": 0.2,
                    "emissive": [0.75, 0.12, 0.10],
                    "emissive_strength": 0.8,
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
                "max_triangles": 7000,
                "max_bones": 64,
                "max_materials": 8,
            },
        },
    },
)
