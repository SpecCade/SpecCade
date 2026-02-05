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
                # Body center
                {"bone": "root", "head": [0.0, 0.0, 0.12], "tail": [0.0, 0.02, 0.12]},
                # Abdomen goes backward (-Y)
                {"bone": "abdomen", "head": [0.0, 0.0, 0.12], "tail": [0.0, -0.20, 0.14], "parent": "root"},
                # Thorax goes forward (+Y)
                {"bone": "thorax", "head": [0.0, 0.0, 0.12], "tail": [0.0, 0.12, 0.12], "parent": "root"},
                # Head goes forward
                {"bone": "head", "head": [0.0, 0.12, 0.12], "tail": [0.0, 0.22, 0.12], "parent": "thorax"},
                # 8 legs - extend outward from thorax, going DOWN to ground
                # Left legs (at -X)
                {"bone": "leg1_l", "head": [-0.06, 0.08, 0.12], "tail": [-0.25, 0.15, 0.0], "parent": "thorax"},
                {"bone": "leg2_l", "head": [-0.06, 0.04, 0.12], "tail": [-0.28, 0.06, 0.0], "parent": "thorax"},
                {"bone": "leg3_l", "head": [-0.06, 0.0, 0.12], "tail": [-0.28, -0.02, 0.0], "parent": "thorax"},
                {"bone": "leg4_l", "head": [-0.06, -0.04, 0.12], "tail": [-0.25, -0.12, 0.0], "parent": "abdomen"},
                # Right legs (at +X) - mirrors
                {"bone": "leg1_r", "head": [0.06, 0.08, 0.12], "tail": [0.25, 0.15, 0.0], "parent": "thorax"},
                {"bone": "leg2_r", "head": [0.06, 0.04, 0.12], "tail": [0.28, 0.06, 0.0], "parent": "thorax"},
                {"bone": "leg3_r", "head": [0.06, 0.0, 0.12], "tail": [0.28, -0.02, 0.0], "parent": "thorax"},
                {"bone": "leg4_r", "head": [0.06, -0.04, 0.12], "tail": [0.25, -0.12, 0.0], "parent": "abdomen"},
            ],
            "bone_meshes": {
                "root": {
                    "profile": "circle(8)",
                    "profile_radius": 0.08,
                    "extrusion_steps": [
                        {"extrude": 1.0, "scale": 1.0},
                    ],
                    "material_index": 0,
                    "connect_end": "bridge",
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
                    "connect_start": "bridge",
                    "connect_end": "bridge",
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
                    "connect_start": "bridge",
                    "connect_end": "bridge",
                },
                "head": {
                    "profile": "circle(12)",
                    "profile_radius": {"absolute": 0.07},
                    "extrusion_steps": [
                        {"extrude": 1.0, "scale": 1.0},
                    ],
                    "material_index": 0,
                    "connect_start": "bridge",
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
                    "profile_radius": 0.035,
                    "extrusion_steps": [
                        {"extrude": 0.50, "scale": 0.85},
                        {"extrude": 0.50, "scale": 0.65},
                    ],
                    "material_index": 0,
                    "cap_start": True,
                },
                "leg2_l": {
                    "profile": "circle(8)",
                    "profile_radius": 0.035,
                    "extrusion_steps": [
                        {"extrude": 0.50, "scale": 0.85},
                        {"extrude": 0.50, "scale": 0.65},
                    ],
                    "material_index": 0,
                    "cap_start": True,
                },
                "leg3_l": {
                    "profile": "circle(8)",
                    "profile_radius": 0.035,
                    "extrusion_steps": [
                        {"extrude": 0.50, "scale": 0.85},
                        {"extrude": 0.50, "scale": 0.65},
                    ],
                    "material_index": 0,
                    "cap_start": True,
                },
                "leg4_l": {
                    "profile": "circle(8)",
                    "profile_radius": 0.035,
                    "extrusion_steps": [
                        {"extrude": 0.50, "scale": 0.85},
                        {"extrude": 0.50, "scale": 0.65},
                    ],
                    "material_index": 0,
                    "cap_start": True,
                },
                "leg1_r": {
                    "profile": "circle(8)",
                    "profile_radius": 0.035,
                    "extrusion_steps": [
                        {"extrude": 0.50, "scale": 0.85},
                        {"extrude": 0.50, "scale": 0.65},
                    ],
                    "material_index": 0,
                    "cap_start": True,
                },
                "leg2_r": {
                    "profile": "circle(8)",
                    "profile_radius": 0.035,
                    "extrusion_steps": [
                        {"extrude": 0.50, "scale": 0.85},
                        {"extrude": 0.50, "scale": 0.65},
                    ],
                    "material_index": 0,
                    "cap_start": True,
                },
                "leg3_r": {
                    "profile": "circle(8)",
                    "profile_radius": 0.035,
                    "extrusion_steps": [
                        {"extrude": 0.50, "scale": 0.85},
                        {"extrude": 0.50, "scale": 0.65},
                    ],
                    "material_index": 0,
                    "cap_start": True,
                },
                "leg4_r": {
                    "profile": "circle(8)",
                    "profile_radius": 0.035,
                    "extrusion_steps": [
                        {"extrude": 0.50, "scale": 0.85},
                        {"extrude": 0.50, "scale": 0.65},
                    ],
                    "material_index": 0,
                    "cap_start": True,
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
