# Armor plate shrinkwrap mesh
#
# Armor chestplate wrapped onto body torso using nearest_surface mode with offset.
# Demonstrates shrinkwrap modifier for character equipment fitting.

spec(
    asset_id = "shrinkwrap-armor-plate",
    asset_type = "static_mesh",
    license = "CC0-1.0",
    seed = 7001,
    description = "Armor chestplate wrapped onto body torso using nearest_surface mode with offset",
    outputs = [output("armor_plate.glb", "glb")],
    recipe = {
        "kind": "static_mesh.shrinkwrap_v1",
        "params": {
            "base_mesh": "primitive://sphere",
            "wrap_mesh": "primitive://cube",
            "mode": "nearest_surface",
            "offset": 0.02,
            "smooth_iterations": 2,
            "smooth_factor": 0.5,
            "validation": {
                "max_self_intersections": 0,
                "min_face_area": 0.0001
            },
            "export": {
                "apply_modifiers": True,
                "triangulate": True,
                "include_normals": True,
                "include_uvs": True,
                "include_vertex_colors": False,
                "tangents": True
            }
        }
    }
)
