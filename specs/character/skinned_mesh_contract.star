# Skinned mesh contract example
#
# Rebinds a generated source mesh to the canonical humanoid_connected_v1
# skeleton using auto weights. The source mesh is generated separately into the
# same out_root by the contract verifier setup phase.

spec(
    asset_id = "skinned_mesh_contract",
    asset_type = "skeletal_mesh",
    seed = 7121,
    license = "CC0-1.0",
    description = "Contract example for skeletal_mesh.skinned_mesh_v1",
    outputs = [output("skeletal_mesh/skinned_mesh_contract.glb", "glb")],
    recipe = {
        "kind": "skeletal_mesh.skinned_mesh_v1",
        "params": {
            "mesh_file": "meshes/skinned_mesh_source.glb",
            "skeleton_preset": "humanoid_connected_v1",
            "binding": {
                "mode": "auto_weights",
                "max_bone_influences": 4
            },
            "material_slots": [
                {
                    "name": "body",
                    "base_color": [0.72, 0.76, 0.82, 1.0],
                    "roughness": 0.65
                }
            ],
            "export": {
                "include_armature": True,
                "include_normals": True,
                "include_uvs": True,
                "triangulate": True,
                "include_skin_weights": True,
                "save_blend": False
            },
            "constraints": {
                "max_triangles": 20000,
                "max_bones": 64,
                "max_materials": 8
            }
        }
    }
)
