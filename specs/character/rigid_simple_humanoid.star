# Simple rigid humanoid - custom skeleton with explicit positioning
#
# Simplified skeleton for testing armature_driven mesh generation.
# All bones defined explicitly to debug positioning issues.

spec(
    asset_id = "rigid_simple_humanoid",
    asset_type = "skeletal_mesh",
    seed = 7400,
    license = "CC0-1.0",
    description = "Simplified rigid humanoid for testing - custom skeleton, minimal bones",
    tags = ["skeletal_mesh", "character", "humanoid", "test"],
    outputs = [output("skeletal_mesh/rigid_simple_humanoid.glb", "glb")],
    recipe = {
        "kind": "skeletal_mesh.armature_driven_v1",
        "params": {
            # Custom skeleton - explicit bone positions
            "skeleton": [
                # Root/pelvis at origin
                {
                    "bone": "root",
                    "head": [0.0, 0.0, 0.0],
                    "tail": [0.0, 0.0, 0.1],
                },
                # Spine - vertical from root
                {
                    "bone": "spine",
                    "head": [0.0, 0.0, 0.1],
                    "tail": [0.0, 0.0, 0.5],
                    "parent": "root",
                },
                # Chest - vertical continuation
                {
                    "bone": "chest",
                    "head": [0.0, 0.0, 0.5],
                    "tail": [0.0, 0.0, 0.8],
                    "parent": "spine",
                },
                # Neck - vertical from chest
                {
                    "bone": "neck",
                    "head": [0.0, 0.0, 0.8],
                    "tail": [0.0, 0.0, 0.95],
                    "parent": "chest",
                },
                # Head - vertical from neck
                {
                    "bone": "head",
                    "head": [0.0, 0.0, 0.95],
                    "tail": [0.0, 0.0, 1.2],
                    "parent": "neck",
                },
                # Left arm - HORIZONTAL from chest center
                # Starts at chest midpoint (z=0.65), extends left (-X)
                {
                    "bone": "arm_l",
                    "head": [-0.15, 0.0, 0.65],
                    "tail": [-0.7, 0.0, 0.65],
                    "parent": "chest",
                },
                # Right arm - mirror
                {
                    "bone": "arm_r",
                    "head": [0.15, 0.0, 0.65],
                    "tail": [0.7, 0.0, 0.65],
                    "parent": "chest",
                },
                # Left leg - vertical down from root
                {
                    "bone": "leg_l",
                    "head": [-0.1, 0.0, 0.0],
                    "tail": [-0.1, 0.0, -0.6],
                    "parent": "root",
                },
                # Right leg - mirror
                {
                    "bone": "leg_r",
                    "head": [0.1, 0.0, 0.0],
                    "tail": [0.1, 0.0, -0.6],
                    "parent": "root",
                },
            ],
            "bone_meshes": {
                "root": {
                    "profile": "circle(8)",
                    "profile_radius": {"absolute": 0.12},
                    "extrusion_steps": [
                        {"extrude": 1.0, "scale": 1.0},
                    ],
                    "cap_start": True,
                    "cap_end": False,
                    "material_index": 0,
                },
                "spine": {
                    "profile": "circle(8)",
                    "profile_radius": {"absolute": 0.12},
                    "extrusion_steps": [
                        {"extrude": 0.5, "scale": 1.1},
                        {"extrude": 0.5, "scale": 0.95},
                    ],
                    "cap_start": False,
                    "cap_end": False,
                    "material_index": 0,
                },
                "chest": {
                    "profile": "circle(8)",
                    "profile_radius": {"absolute": 0.14},
                    "extrusion_steps": [
                        {"extrude": 0.3, "scale": 1.15},
                        {"extrude": 0.4, "scale": 1.0},
                        {"extrude": 0.3, "scale": 0.8},
                    ],
                    "cap_start": False,
                    "cap_end": False,
                    "material_index": 0,
                },
                "neck": {
                    "profile": "circle(8)",
                    "profile_radius": {"absolute": 0.06},
                    "extrusion_steps": [
                        {"extrude": 1.0, "scale": 0.9},
                    ],
                    "cap_start": True,
                    "cap_end": False,
                    "material_index": 0,
                },
                "head": {
                    "profile": "circle(8)",
                    "profile_radius": {"absolute": 0.1},
                    "extrusion_steps": [
                        {"extrude": 0.3, "scale": 1.3},
                        {"extrude": 0.5, "scale": 1.0},
                        {"extrude": 0.2, "scale": 0.7},
                    ],
                    "cap_start": False,
                    "cap_end": True,
                    "material_index": 0,
                },
                "arm_l": {
                    "profile": "circle(8)",
                    "profile_radius": {"absolute": 0.10},
                    "extrusion_steps": [
                        {"extrude": 0.3, "scale": 1.0},
                        {"extrude": 0.5, "scale": 0.85},
                        {"extrude": 0.2, "scale": 0.7},
                    ],
                    "cap_start": True,
                    "cap_end": True,
                    "material_index": 0,
                },
                "arm_r": {"mirror": "arm_l"},
                "leg_l": {
                    "profile": "circle(8)",
                    "profile_radius": {"absolute": 0.07},
                    "extrusion_steps": [
                        {"extrude": 0.3, "scale": 1.0},
                        {"extrude": 0.5, "scale": 0.8},
                        {"extrude": 0.2, "scale": 0.6},
                    ],
                    "cap_start": True,
                    "cap_end": True,
                    "material_index": 0,
                },
                "leg_r": {"mirror": "leg_l"},
            },
            "material_slots": [
                {
                    "name": "body",
                    "base_color": [0.7, 0.7, 0.75, 1.0],
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
            "constraints": {
                "max_triangles": 5000,
                "max_bones": 16,
                "max_materials": 2,
            },
        },
    },
)
