# Fitted accessory shrinkwrap mesh
#
# Fitted accessory (ring, bracelet, belt) wrapped onto curved surface using
# nearest_vertex for stylized fit. Demonstrates shrinkwrap for jewelry/accessories.

spec(
    asset_id = "shrinkwrap-fitted-accessory",
    asset_type = "static_mesh",
    license = "CC0-1.0",
    seed = 7003,
    description = "Fitted accessory (ring, bracelet, belt) wrapped onto curved surface using nearest_vertex for stylized fit",
    outputs = [output("fitted_accessory.glb", "glb")],
    recipe = {
        "kind": "static_mesh.shrinkwrap_v1",
        "params": {
            "base_mesh": "primitive://torus",
            "wrap_mesh": "primitive://cube",
            "mode": "nearest_vertex",
            "offset": 0.005,
            "smooth_iterations": 1,
            "smooth_factor": 0.4,
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
                "tangents": False
            }
        }
    }
)
