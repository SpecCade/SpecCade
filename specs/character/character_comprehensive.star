# Comprehensive character (golden fixture) - exercises most armature-driven fields

spec(
    asset_id = "character_comprehensive",
    asset_type = "skeletal_mesh",
    seed = 7200,
    license = "CC0-1.0",
    description = "Golden fixture: comprehensive armature-driven skeletal mesh exercising bool_shapes, modifiers, attachments, materials, export, constraints",
    tags = ["golden", "skeletal_mesh", "character", "comprehensive", "armature_driven_v1"],
    outputs = [output("skeletal_mesh/character_comprehensive.glb", "glb")],
    recipe = {
        "kind": "skeletal_mesh.armature_driven_v1",
        "params": {
            "skeleton_preset": "humanoid_basic_v1",
            "material_slots": [
                {
                    "name": "skin",
                    "base_color": [0.82, 0.68, 0.58, 1.0],
                    "roughness": 0.55,
                },
                {
                    "name": "cloth",
                    "base_color": [0.14, 0.16, 0.18, 1.0],
                    "roughness": 0.9,
                },
                {
                    "name": "metal",
                    "base_color": [0.55, 0.56, 0.58, 1.0],
                    "metallic": 1.0,
                    "roughness": 0.25,
                    "emissive": [0.2, 0.25, 0.3],
                    "emissive_strength": 0.5,
                },
            ],
            "bool_shapes": {
                "eye_cut_l": {
                    "primitive": "sphere",
                    "dimensions": [0.07, 0.07, 0.07],
                    "position": [-0.18, 0.22, 0.14],
                    "bone": "head",
                },
                "eye_cut_r": {
                    "mirror": "eye_cut_l",
                },
                "mouth_cut": {
                    "primitive": "cube",
                    "dimensions": [0.22, 0.12, 0.08],
                    "position": [0.0, 0.30, -0.05],
                    "bone": "head",
                },
                "chest_notch": {
                    "primitive": "cylinder",
                    "dimensions": [0.14, 0.14, 0.40],
                    "position": [0.0, 0.20, 0.05],
                    "bone": "chest",
                },
            },
            "bone_meshes": {
                "spine": {
                    "profile": "circle(12)",
                    "profile_radius": 0.11,
                    "translate": [0.0, 0.0, 0.05],
                    "rotate": [0.0, 0.0, 10.0],
                    "extrusion_steps": [
                        {"extrude": 0.15, "scale": 1.15, "rotate": 3.0},
                        {"extrude": 0.45, "scale": 0.95, "rotate": 6.0},
                        {"extrude": 0.40, "scale": 0.95, "rotate": 3.0},
                    ],
                    "cap_start": True,
                    "cap_end": False,
                    "material_index": 1,
                    "modifiers": [
                        {
                            "bevel": {
                                "width": 0.02,
                                "segments": 2,
                            },
                        },
                        {
                            "subdivide": {
                                "cuts": 1,
                            },
                        },
                        {
                            "bool": {
                                "operation": "difference",
                                "target": "chest_notch",
                            },
                        },
                    ],
                    "attachments": [
                        {
                            "primitive": "cube",
                            "dimensions": [0.15, 0.20, 0.08],
                            "offset": [0.0, 0.25, -0.05],
                            "rotation": [0.0, 0.0, 25.0],
                            "material_index": 2,
                        },
                        {
                            "extrude": {
                                "profile": "rectangle",
                                "start": [0.0, 0.05, 0.0],
                                "end": [0.0, 0.45, 0.0],
                                "profile_radius": [0.05, 0.09],
                                "taper": 0.7,
                            },
                        },
                        {
                            "primitive": "cone",
                            "dimensions": [0.16, 0.16, 0.22],
                            "offset": [0.0, 0.05, 0.65],
                            "rotation": [0.0, 0.0, 0.0],
                            "material_index": 1,
                        },
                    ],
                },
                "chest": {
                    "profile": "hexagon(6)",
                    "profile_radius": [0.14, 0.11],
                    "translate": [0.0, 0.03, 0.0],
                    "rotate": [-4.0, 0.0, 0.0],
                    "extrusion_steps": [
                        {"extrude": 0.25, "scale": 1.2, "rotate": -2.0},
                        {"extrude": 0.60, "scale": 0.8, "rotate": -4.0},
                        {"extrude": 0.15, "scale": 0.85, "rotate": -2.0},
                    ],
                    "cap_start": False,
                    "cap_end": False,
                    "material_index": 1,
                    "modifiers": [
                        {
                            "bevel": {
                                "width": 0.015,
                                "segments": 1,
                            },
                        },
                    ],
                },
                "head": {
                    "profile": "circle(16)",
                    "profile_radius": {"absolute": 0.11},
                    "translate": [0.0, 0.08, 0.0],
                    "rotate": [0.0, 0.0, 0.0],
                    "extrusion_steps": [
                        {"extrude": 0.05, "scale": 1.15},
                        {"extrude": 0.50, "scale": 1.25},
                        {"extrude": 0.40, "scale": 0.85},
                        {"extrude": 0.05, "scale": 1.0},
                    ],
                    "cap_start": True,
                    "cap_end": True,
                    "material_index": 0,
                    "modifiers": [
                        {
                            "subdivide": {
                                "cuts": 1,
                            },
                        },
                        {
                            "bool": {
                                "operation": "difference",
                                "target": "eye_cut_l",
                            },
                        },
                        {
                            "bool": {
                                "operation": "difference",
                                "target": "eye_cut_r",
                            },
                        },
                        {
                            "bool": {
                                "operation": "difference",
                                "target": "mouth_cut",
                            },
                        },
                    ],
                    "attachments": [
                        {
                            "primitive": "ico_sphere",
                            "dimensions": [0.06, 0.06, 0.06],
                            "offset": [0.0, 0.38, 0.15],
                            "rotation": [0.0, 0.0, 0.0],
                            "material_index": 2,
                        },
                    ],
                },
                "upper_arm_l": {
                    "profile": "rectangle",
                    "profile_radius": [0.08, 0.12],
                    "translate": [0.0, 0.0, 0.02],
                    "rotate": [0.0, 0.0, -15.0],
                    "extrusion_steps": [
                        {"extrude": 0.20, "scale": 1.15, "rotate": 8.0},
                        {"extrude": 0.50, "scale": 0.9, "rotate": 12.0},
                        {"extrude": 0.30, "scale": 0.8, "rotate": 5.0},
                    ],
                    "cap_start": False,
                    "cap_end": False,
                    "material_index": 0,
                    "modifiers": [
                        {
                            "bevel": {
                                "width": 0.01,
                                "segments": 2,
                            },
                        },
                        {
                            "bool": {
                                "operation": "intersect",
                                "target": "chest_notch",
                            },
                        },
                    ],
                    "attachments": [
                        {
                            "primitive": "torus",
                            "dimensions": [0.18, 0.18, 0.08],
                            "offset": [0.0, 0.35, 0.0],
                            "rotation": [90.0, 0.0, 0.0],
                            "material_index": 2,
                        },
                    ],
                },
                "upper_arm_r": {
                    "mirror": "upper_arm_l",
                },
                "lower_arm_l": {
                    "profile": "hexagon(8)",
                    "profile_radius": 0.07,
                    "translate": [0.0, 0.02, 0.0],
                    "rotate": [0.0, 10.0, 0.0],
                    "extrusion_steps": [
                        {"extrude": 0.35, "scale": 1.1, "rotate": -6.0},
                        {"extrude": 0.65, "scale": 0.75, "rotate": -12.0},
                    ],
                    "cap_start": False,
                    "cap_end": False,
                    "material_index": 0,
                    "modifiers": [
                        {
                            "subdivide": {
                                "cuts": 2,
                            },
                        },
                        {
                            "bool": {
                                "operation": "union",
                                "target": "eye_cut_l",
                            },
                        },
                    ],
                    "attachments": [
                        {
                            "primitive": "cone",
                            "dimensions": [0.08, 0.08, 0.14],
                            "offset": [0.0, 0.45, 0.02],
                            "rotation": [0.0, 0.0, 0.0],
                            "material_index": 2,
                        },
                    ],
                },
                "lower_arm_r": {
                    "mirror": "lower_arm_l",
                },
            },
            "export": {
                "include_armature": True,
                "include_normals": True,
                "include_uvs": True,
                "triangulate": True,
                "include_skin_weights": True,
                "save_blend": False,
            },
            "constraints": {
                "max_triangles": 20000,
                "max_bones": 64,
                "max_materials": 8,
            },
        },
    },
)
