# Adversarial: Character with no bones
# Expected: validation rejects empty skeleton

spec(
    asset_id = "adv-empty-skeleton",
    asset_type = "skeletal_mesh",
    license = "CC0-1.0",
    seed = 99907,
    outputs = [output("meshes/empty_skel.glb", "glb")],
    recipe = {
        "kind": "skeletal_mesh.armature_driven_v1",
        "params": {
            "skeleton": [],
            "bone_meshes": {}
        }
    }
)
