# Cloth drape shrinkwrap mesh
#
# Draped cloth/cape wrapped onto body using project mode for natural draping effect.
# Demonstrates shrinkwrap modifier for cloth and fabric simulation.

spec(
    asset_id = "shrinkwrap-cloth-drape",
    asset_type = "static_mesh",
    license = "CC0-1.0",
    seed = 7002,
    description = "Draped cloth/cape wrapped onto body using project mode for natural draping effect",
    outputs = [output("cloth_drape.glb", "glb")],
    recipe = {
        "kind": "static_mesh.shrinkwrap_v1",
        "params": {
            "base_mesh": "primitive://cylinder",
            "wrap_mesh": "primitive://plane",
            "mode": "project",
            "offset": 0.01,
            "smooth_iterations": 3,
            "smooth_factor": 0.6,
            "validation": {
                "max_self_intersections": 0,
                "min_face_area": 0.00005
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
