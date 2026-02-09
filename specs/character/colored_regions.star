# Character with colored texture regions - demonstrates region colors and advanced UV modes

spec(
    asset_id = "colored_regions",
    asset_type = "skeletal_mesh",
    license = "CC0-1.0",
    seed = 7006,
    description = "Character with colored texture regions - demonstrates region colors and advanced UV modes",
    outputs = [output("colored_regions.glb", "glb")],
    recipe = {
        "kind": "skeletal_mesh.armature_driven_v1",
        "params": {
            "skeleton": [
                {"bone": "root", "head": [0, 0, 0], "tail": [0, 0, 0.1]},
                {"bone": "torso", "head": [0, 0, 0.1], "tail": [0, 0, 0.5], "parent": "root"},
                {"bone": "head", "head": [0, 0, 0.5], "tail": [0, 0, 0.7], "parent": "torso"},
                # Shoulder stubs connect torso to arms
                {"bone": "shoulder_L", "head": [-0.15, 0, 0.45], "tail": [-0.22, 0, 0.45], "parent": "torso"},
                {"bone": "shoulder_R", "head": [0.15, 0, 0.45], "tail": [0.22, 0, 0.45], "parent": "torso"},
                {"bone": "arm_L", "head": [-0.22, 0, 0.45], "tail": [-0.55, 0, 0.45], "parent": "shoulder_L"},
                {"bone": "arm_R", "head": [0.22, 0, 0.45], "tail": [0.55, 0, 0.45], "parent": "shoulder_R"},
                {"bone": "leg_L", "head": [-0.08, 0, 0.1], "tail": [-0.08, 0, -0.3], "parent": "root"},
                {"bone": "leg_R", "head": [0.08, 0, 0.1], "tail": [0.08, 0, -0.3], "parent": "root"},
                # Feet extend forward from leg bottoms
                {"bone": "foot_L", "head": [-0.08, 0, -0.3], "tail": [-0.08, 0.12, -0.3], "parent": "leg_L"},
                {"bone": "foot_R", "head": [0.08, 0, -0.3], "tail": [0.08, 0.12, -0.3], "parent": "leg_R"},
            ],
            "bone_meshes": {
                "root": {
                    "profile": "circle(8)",
                    "profile_radius": {"absolute": 0.12},
                    "cap_start": True,
                    "material_index": 0,
                },
                "torso": {
                    "profile": "circle(8)",
                    "profile_radius": {"absolute": 0.14},
                    "extrusion_steps": [
                        {"extrude": 0.4, "scale": 1.1},
                        {"extrude": 0.3, "scale": 1.0},
                        {"extrude": 0.3, "scale": 0.85},
                    ],
                    "material_index": 1,
                },
                "head": {
                    "profile": "circle(8)",
                    "profile_radius": {"absolute": 0.1},
                    "extrusion_steps": [
                        {"extrude": 0.3, "scale": 1.3},
                        {"extrude": 0.5, "scale": 1.0},
                        {"extrude": 0.2, "scale": 0.7},
                    ],
                    "cap_start": True,
                    "cap_end": True,
                    "material_index": 2,
                },
                "shoulder_L": {
                    "profile": "circle(8)",
                    "profile_radius": {"absolute": 0.08},
                    "cap_start": True,
                    "material_index": 1,
                },
                "shoulder_R": {"mirror": "shoulder_L"},
                "arm_L": {
                    "profile": "circle(8)",
                    "profile_radius": {"absolute": 0.07},
                    "extrusion_steps": [
                        {"extrude": 0.5, "scale": 0.9},
                        {"extrude": 0.5, "scale": 0.75},
                    ],
                    "cap_end": True,
                    "material_index": 3,
                },
                "arm_R": {"mirror": "arm_L"},
                "leg_L": {
                    "profile": "circle(8)",
                    "profile_radius": {"absolute": 0.07},
                    "extrusion_steps": [
                        {"extrude": 0.4, "scale": 1.0},
                        {"extrude": 0.4, "scale": 0.85},
                        {"extrude": 0.2, "scale": 0.75},
                    ],
                    "cap_start": True,
                    "material_index": 4,
                },
                "leg_R": {"mirror": "leg_L"},
                "foot_L": {
                    "profile": "circle(8)",
                    "profile_radius": {"absolute": 0.06},
                    "cap_start": True,
                    "cap_end": True,
                    "material_index": 0,
                },
                "foot_R": {"mirror": "foot_L"},
            },
            "material_slots": [
                {
                    "name": "hips_feet",
                    "base_color": [0.2, 0.2, 0.8, 1.0],
                    "roughness": 0.5,
                },
                {
                    "name": "torso_shoulders",
                    "base_color": [0.8, 0.2, 0.2, 1.0],
                    "roughness": 0.5,
                },
                {
                    "name": "head",
                    "base_color": [0.9, 0.75, 0.55, 1.0],
                    "roughness": 0.4,
                },
                {
                    "name": "arms",
                    "base_color": [0.2, 0.7, 0.2, 1.0],
                    "roughness": 0.5,
                },
                {
                    "name": "legs",
                    "base_color": [0.6, 0.4, 0.8, 1.0],
                    "roughness": 0.5,
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
            "constraints": {"max_triangles": 5000, "max_materials": 8},
        }
    }
)
