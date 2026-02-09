# Humanoid female (golden fixture) - armature-driven using humanoid_connected_v1 preset

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
            "skeleton_preset": "humanoid_connected_v1",
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
                    "profile_radius": {"absolute": 0.105},
                    "extrusion_steps": [
                        # 0% -> 35%: slight bulge to 1.08
                        {"extrude": 0.35, "scale": 1.08},
                        # 35% -> 100%: taper to 0.96
                        {"extrude": 0.65, "scale": 0.96},
                    ],
                    "material_index": 1,
                },
                "chest": {
                    "profile": "hexagon(8)",
                    "profile_radius": {"absolute": 0.13},
                    "translate": [0.0, 0.04, 0.0],
                    "rotate": [-2.0, 0.0, 0.0],
                    "extrusion_steps": [
                        # 0% -> 30%: chest bulge to 1.18
                        {"extrude": 0.30, "scale": 1.18},
                        # 30% -> 86%: taper to 0.88
                        {"extrude": 0.56, "scale": 0.88},
                        # 86% -> 100%: final taper to 0.90
                        {"extrude": 0.14, "scale": 0.90},
                    ],
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
                    "extrusion_steps": [
                        # 0% -> 10%: slight swell to 1.08
                        {"extrude": 0.10, "scale": 1.08},
                        # 10% -> 55%: bulge to 1.22 (cranium)
                        {"extrude": 0.45, "scale": 1.22},
                        # 55% -> 100%: taper to 1.0 (top of head)
                        {"extrude": 0.45, "scale": 1.0},
                    ],
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
                # Shoulder bones connect chest to upper arms
                "shoulder_l": {
                    "profile": "circle(10)",
                    "profile_radius": {"absolute": 0.07},
                    "extrusion_steps": [
                        {"extrude": 0.4, "scale": 1.08},
                        {"extrude": 0.6, "scale": 0.92},
                    ],
                    "material_index": 0,
                    "cap_start": True,
                    "cap_end": False,
                },
                "shoulder_r": {"mirror": "shoulder_l"},
                "upper_arm_l": {
                    "profile": "circle(10)",
                    "profile_radius": {"absolute": 0.065},
                    "extrusion_steps": [
                        # 0% -> 18%: shoulder bulge to 1.12
                        {"extrude": 0.18, "scale": 1.12},
                        # 18% -> 70%: taper to 0.92
                        {"extrude": 0.52, "scale": 0.92},
                        # 70% -> 100%: final taper to 0.82
                        {"extrude": 0.30, "scale": 0.82},
                    ],
                    "material_index": 0,
                    "cap_start": False,
                    "cap_end": False,
                },
                "upper_arm_r": {"mirror": "upper_arm_l"},
                "lower_arm_l": {
                    "profile": "circle(10)",
                    "profile_radius": {"absolute": 0.048},
                    "extrusion_steps": [
                        {"extrude": 0.20, "scale": 1.05},
                        {"extrude": 0.55, "scale": 0.88},
                        {"extrude": 0.25, "scale": 0.80},
                    ],
                    "material_index": 0,
                    "cap_start": False,
                    "cap_end": False,
                },
                "lower_arm_r": {"mirror": "lower_arm_l"},
                "hand_l": {
                    "profile": "circle(10)",
                    "profile_radius": {"absolute": 0.035},
                    "extrusion_steps": [
                        {"extrude": 0.25, "scale": 1.3},
                        {"extrude": 0.50, "scale": 1.1},
                        {"extrude": 0.25, "scale": 0.6},
                    ],
                    "material_index": 0,
                    "cap_start": False,
                    "cap_end": True,
                },
                "hand_r": {"mirror": "hand_l"},
                "upper_leg_l": {
                    "profile": "circle(10)",
                    "profile_radius": {"absolute": 0.125},
                    "extrusion_steps": [
                        # 0% -> 20%: hip bulge to 1.12
                        {"extrude": 0.20, "scale": 1.12},
                        # 20% -> 65%: taper to 0.95
                        {"extrude": 0.45, "scale": 0.95},
                        # 65% -> 100%: final taper to 0.75
                        {"extrude": 0.35, "scale": 0.75},
                    ],
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
