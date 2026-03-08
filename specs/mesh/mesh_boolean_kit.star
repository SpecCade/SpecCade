# Boolean kit hard-surface mesh
#
# Demonstrates the canonical boolean kit pipeline with a simple hard-surface
# blockout and explicit cleanup settings.

spec(
    asset_id = "stdlib-boolean-kit-panel-01",
    asset_type = "static_mesh",
    seed = 7050,
    license = "CC0-1.0",
    description = "Hard-surface control panel built with boolean kit operations",
    outputs = [output("meshes/boolean_kit_panel.glb", "glb")],
    recipe = {
        "kind": "static_mesh.boolean_kit_v1",
        "params": {
            "base": {
                "primitive": "cube",
                "dimensions": [1.6, 0.5, 0.25]
            },
            "operations": [
                {
                    "op": "difference",
                    "target": {
                        "primitive": "cube",
                        "dimensions": [0.9, 0.3, 0.12],
                        "position": [0.0, 0.0, 0.04]
                    }
                },
                {
                    "op": "union",
                    "target": {
                        "primitive": "cylinder",
                        "dimensions": [0.18, 0.18, 0.28],
                        "position": [-0.45, 0.0, 0.0],
                        "rotation": [90.0, 0.0, 0.0]
                    }
                },
                {
                    "op": "union",
                    "target": {
                        "primitive": "cylinder",
                        "dimensions": [0.18, 0.18, 0.28],
                        "position": [0.45, 0.0, 0.0],
                        "rotation": [90.0, 0.0, 0.0]
                    }
                }
            ],
            "cleanup": {
                "merge_distance": 0.001,
                "remove_doubles": True,
                "recalc_normals": True,
                "fill_holes": False,
                "dissolve_degenerate": True
            },
            "export": {
                "apply_modifiers": True,
                "triangulate": True,
                "include_normals": True,
                "include_uvs": True
            }
        }
    }
)
