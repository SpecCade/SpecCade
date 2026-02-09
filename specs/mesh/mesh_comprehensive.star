# Comprehensive static mesh fixture - exercises optional mesh recipe fields

spec(
    asset_id = "mesh_comprehensive",
    asset_type = "static_mesh",
    license = "CC0-1.0",
    seed = 4243,
    description = "Comprehensive static mesh spec (modifiers, UVs, normals, materials, LOD, collision, navmesh, baking, attachments)",
    outputs = [output("meshes/mesh_comprehensive.glb", "glb")],
    recipe = {
        "kind": "static_mesh.blender_primitives_v1",
        "params": {
            "base_primitive": "cylinder",
            "dimensions": [1.2, 1.2, 2.2],
            "modifiers": [
                bevel_modifier(0.03, 2, 35.0),
                subdivision_modifier(1, 1),
                mirror_modifier(True, False, False),
                array_modifier(2, [0.0, 1.6, 0.0]),
                solidify_modifier(0.04, 0.0),
                decimate_modifier(0.85),
                triangulate_modifier("clip", "fixed"),
            ],
            "uv_projection": {
                "method": "smart",
                "angle_limit": 66.0,
                "texel_density": 512.0,
                "uv_margin": 0.001,
                "lightmap_uv": True,
            },
            "normals": {
                "preset": "weighted_normals",
                "keep_sharp": True,
            },
            "material_slots": [
                {
                    "name": "painted_metal",
                    "base_color": [0.15, 0.16, 0.18, 1.0],
                    "metallic": 0.7,
                    "roughness": 0.35,
                },
                {
                    "name": "rubber",
                    "base_color": [0.03, 0.03, 0.035, 1.0],
                    "metallic": 0.0,
                    "roughness": 0.9,
                },
                {
                    "name": "warning_stripe",
                    "base_color": [0.9, 0.75, 0.1, 1.0],
                    "roughness": 0.5,
                },
                {
                    "name": "status_led",
                    "emissive": [0.2, 1.0, 0.3],
                    "emissive_strength": 3.0,
                },
            ],
            "lod_chain": {
                "levels": [
                    {"level": 0},
                    {"level": 1, "target_tris": 1200},
                    {"level": 2, "target_tris": 400},
                ],
                "decimate_method": "planar",
            },
            "collision_mesh": {
                "collision_type": "simplified_mesh",
                "target_faces": 128,
                "output_suffix": "_col",
            },
            "navmesh": {
                "walkable_slope_max": 50.0,
                "stair_detection": True,
                "stair_step_height": 0.25,
            },
            "baking": {
                "bake_types": ["normal", "ao", "curvature"],
                "ray_distance": 0.1,
                "margin": 16,
                "resolution": [512, 512],
            },
            "attachments": [
                {
                    "primitive": "torus",
                    "dimensions": [0.8, 0.8, 0.25],
                    "position": [0.0, 0.0, 1.2],
                    "rotation": [0.0, 90.0, 0.0],
                },
                {
                    "primitive": "cone",
                    "dimensions": [0.4, 0.4, 0.7],
                    "position": [0.0, 0.9, 0.3],
                    "rotation": [-90.0, 0.0, 0.0],
                },
                {
                    "primitive": "cube",
                    "dimensions": [0.2, 0.6, 0.2],
                    "position": [0.0, -0.8, 0.8],
                    "rotation": [0.0, 0.0, 0.0],
                },
            ],
            "export": {
                "apply_modifiers": True,
                "triangulate": True,
                "include_normals": True,
                "include_uvs": True,
                "include_vertex_colors": False,
                "tangents": True,
                "save_blend": False,
            },
            "constraints": {
                "max_triangles": 100000,
                "max_vertices": 100000,
                "max_materials": 8,
            },
        },
    },
)
