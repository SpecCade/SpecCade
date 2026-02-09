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
                # Root at body center
                {
                    "bone": "root",
                    "head": [0.0, 0.0, 0.35],
                    "tail": [0.0, 0.05, 0.35],
                },
                # Spine/body runs along +Y (horizontal)
                {
                    "bone": "spine_rear",
                    "head": [0.0, -0.15, 0.35],
                    "tail": [0.0, 0.0, 0.35],
                    "parent": "root",
                },
                {
                    "bone": "spine_front",
                    "head": [0.0, 0.0, 0.35],
                    "tail": [0.0, 0.15, 0.35],
                    "parent": "root",
                },
                {
                    "bone": "chest",
                    "head": [0.0, 0.15, 0.35],
                    "tail": [0.0, 0.25, 0.35],
                    "parent": "spine_front",
                },
                # Neck goes up and forward
                {
                    "bone": "neck",
                    "head": [0.0, 0.25, 0.35],
                    "tail": [0.0, 0.32, 0.45],
                    "parent": "chest",
                },
                # Head goes forward (horizontal)
                {
                    "bone": "head",
                    "head": [0.0, 0.32, 0.45],
                    "tail": [0.0, 0.50, 0.45],
                    "parent": "neck",
                },
                # Tail goes backward (-Y) from spine_rear tail
                {
                    "bone": "tail",
                    "head": [0.0, -0.15, 0.35],
                    "tail": [0.0, -0.32, 0.45],
                    "parent": "spine_rear",
                },
                # Front left leg - goes DOWN from chest, outside body radius
                {
                    "bone": "front_leg_l",
                    "head": [-0.10, 0.18, 0.35],
                    "tail": [-0.10, 0.18, 0.0],
                    "parent": "chest",
                },
                # Front right leg
                {
                    "bone": "front_leg_r",
                    "head": [0.10, 0.18, 0.35],
                    "tail": [0.10, 0.18, 0.0],
                    "parent": "chest",
                },
                # Hind left leg - goes DOWN from rear spine, outside body radius
                {
                    "bone": "hind_leg_l",
                    "head": [-0.10, -0.10, 0.35],
                    "tail": [-0.10, -0.10, 0.0],
                    "parent": "spine_rear",
                },
                # Hind right leg
                {
                    "bone": "hind_leg_r",
                    "head": [0.10, -0.10, 0.35],
                    "tail": [0.10, -0.10, 0.0],
                    "parent": "spine_rear",
                },
            ],
            "bone_meshes": {
                "root": {
                    "profile": "circle(12)",
                    "profile_radius": 0.10,
                    "extrusion_steps": [
                        {"extrude": 1.0, "scale": 1.0},
                    ],
                    "material_index": 0,
                    "connect_end": "bridge",
                },
                "spine_rear": {
                    "profile": "circle(12)",
                    "profile_radius": 0.11,
                    "extrusion_steps": [
                        # Rear section: belly bulge
                        {"extrude": 0.40, "scale": 1.15},
                        {"extrude": 0.60, "scale": 1.0},
                    ],
                    "material_index": 0,
                    "connect_start": "bridge",
                },
                "spine_front": {
                    "profile": "circle(12)",
                    "profile_radius": 0.11,
                    "extrusion_steps": [
                        {"extrude": 0.50, "scale": 0.95},
                        {"extrude": 0.50, "scale": 0.88},
                    ],
                    "material_index": 0,
                    "connect_start": "bridge",
                    "connect_end": "bridge",
                },
                "chest": {
                    "profile": "circle(12)",
                    "profile_radius": 0.12,
                    "extrusion_steps": [
                        {"extrude": 0.50, "scale": 0.95},
                        {"extrude": 0.50, "scale": 0.85},
                    ],
                    "material_index": 0,
                    "connect_start": "bridge",
                    "connect_end": "bridge",
                },
                "neck": {
                    "profile": "circle(12)",
                    "profile_radius": 0.08,
                    "extrusion_steps": [
                        {"extrude": 0.50, "scale": 0.92},
                        {"extrude": 0.50, "scale": 0.85},
                    ],
                    "material_index": 0,
                    "connect_start": "bridge",
                    "connect_end": "bridge",
                },
                "head": {
                    "profile": "circle(12)",
                    "profile_radius": {"absolute": 0.065},
                    "extrusion_steps": [
                        {"extrude": 0.3, "scale": 1.15},
                        {"extrude": 0.5, "scale": 1.0},
                        {"extrude": 0.2, "scale": 0.75},
                    ],
                    "material_index": 0,
                    "connect_start": "bridge",
                    "cap_end": True,
                    "attachments": [
                        {
                            "primitive": "cone",
                            "dimensions": [0.06, 0.06, 0.10],
                            "offset": [-0.06, 0.08, 0.06],
                            "rotation": [0.0, 0.0, -20.0],
                            "material_index": 1,
                        },
                        {
                            "primitive": "cone",
                            "dimensions": [0.06, 0.06, 0.10],
                            "offset": [0.06, 0.08, 0.06],
                            "rotation": [0.0, 0.0, 20.0],
                            "material_index": 1,
                        },
                    ],
                },
                "tail": {
                    "profile": "circle(8)",
                    "profile_radius": {"absolute": 0.025},
                    "extrusion_steps": [
                        {"extrude": 0.50, "scale": 0.80},
                        {"extrude": 0.50, "scale": 0.40},
                    ],
                    "material_index": 0,
                    "cap_start": True,
                    "cap_end": True,
                },
                "front_leg_l": {
                    "profile": "circle(8)",
                    "profile_radius": {"absolute": 0.035},
                    "extrusion_steps": [
                        {"extrude": 0.30, "scale": 0.92},
                        {"extrude": 0.40, "scale": 0.82},
                        {"extrude": 0.30, "scale": 0.95},
                    ],
                    "material_index": 0,
                    "cap_start": True,
                    "cap_end": True,
                },
                "front_leg_r": {
                    "profile": "circle(8)",
                    "profile_radius": {"absolute": 0.035},
                    "extrusion_steps": [
                        {"extrude": 0.30, "scale": 0.92},
                        {"extrude": 0.40, "scale": 0.82},
                        {"extrude": 0.30, "scale": 0.95},
                    ],
                    "material_index": 0,
                    "cap_start": True,
                    "cap_end": True,
                },
                "hind_leg_l": {
                    "profile": "circle(8)",
                    "profile_radius": {"absolute": 0.04},
                    "extrusion_steps": [
                        {"extrude": 0.30, "scale": 0.93},
                        {"extrude": 0.40, "scale": 0.80},
                        {"extrude": 0.30, "scale": 0.95},
                    ],
                    "material_index": 0,
                    "cap_start": True,
                    "cap_end": True,
                },
                "hind_leg_r": {
                    "profile": "circle(8)",
                    "profile_radius": {"absolute": 0.04},
                    "extrusion_steps": [
                        {"extrude": 0.30, "scale": 0.93},
                        {"extrude": 0.40, "scale": 0.80},
                        {"extrude": 0.30, "scale": 0.95},
                    ],
                    "material_index": 0,
                    "cap_start": True,
                    "cap_end": True,
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
