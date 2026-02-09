# Environment house - architectural static mesh fixture

spec(
    asset_id = "environment_house",
    asset_type = "static_mesh",
    license = "CC0-1.0",
    seed = 4242,
    description = "Simple environment house blockout (body + roof + details) for static-mesh golden coverage",
    outputs = [output("meshes/environment_house.glb", "glb")],
    recipe = {
        "kind": "static_mesh.blender_primitives_v1",
        "params": {
            "base_primitive": "cube",
            "dimensions": [3.0, 4.0, 2.5],
            "modifiers": [
                bevel_modifier(0.04, 3),
                edge_split_modifier(40.0),
                triangulate_modifier("beauty", "shortest_diagonal"),
            ],
            "uv_projection": {
                "method": "smart",
                "angle_limit": 66.0,
                "texel_density": 256.0,
                "uv_margin": 0.002,
                "lightmap_uv": True,
            },
            "normals": {
                "preset": "auto_smooth",
                "angle": 45.0,
                "keep_sharp": True,
            },
            "material_slots": [
                {
                    "name": "house_walls",
                    "base_color": [0.85, 0.82, 0.78, 1.0],
                    "roughness": 0.85,
                },
                {
                    "name": "roof_tiles",
                    "base_color": [0.45, 0.15, 0.12, 1.0],
                    "roughness": 0.7,
                },
                {
                    "name": "window_glass",
                    "base_color": [0.2, 0.25, 0.3, 0.4],
                    "metallic": 0.0,
                    "roughness": 0.05,
                },
                {
                    "name": "porch_light",
                    "emissive": [1.0, 0.85, 0.5],
                    "emissive_strength": 2.0,
                },
            ],
            "attachments": [
                # Roof — cone sized to match wall footprint (3.0 x 4.0)
                {
                    "primitive": "cone",
                    "dimensions": [3.4, 4.4, 2.0],
                    "position": [0.0, 0.0, 2.25],
                    "rotation": [0.0, 0.0, 0.0],
                },
                # Chimney
                {
                    "primitive": "cube",
                    "dimensions": [0.5, 0.5, 1.0],
                    "position": [1.2, -1.1, 2.2],
                    "rotation": [0.0, 0.0, 0.0],
                },
                # Door frame bump
                {
                    "primitive": "cube",
                    "dimensions": [0.9, 0.1, 1.8],
                    "position": [0.0, 2.05, 0.1],
                    "rotation": [0.0, 0.0, 0.0],
                },
                # Window frame bump
                {
                    "primitive": "cube",
                    "dimensions": [0.8, 0.1, 0.6],
                    "position": [-1.1, 2.05, 0.6],
                    "rotation": [0.0, 0.0, 0.0],
                },
                # Porch light block
                {
                    "primitive": "cube",
                    "dimensions": [0.15, 0.15, 0.15],
                    "position": [-1.1, 2.15, 0.6],
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
                "max_triangles": 50000,
                "max_vertices": 50000,
                "max_materials": 8,
            },
        },
    },
)
