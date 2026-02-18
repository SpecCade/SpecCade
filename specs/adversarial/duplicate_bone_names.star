# Adversarial: Character with bone_meshes referencing unknown bone name
# Expected: validation rejects bone_meshes key for a non-existent bone

spec(
    asset_id = "adv-unknown-bone-mesh",
    asset_type = "skeletal_mesh",
    license = "CC0-1.0",
    seed = 99908,
    outputs = [output("meshes/bad_bones.glb", "glb")],
    recipe = {
        "kind": "skeletal_mesh.armature_driven_v1",
        "params": {
            "skeleton": [
                {"bone": "root", "head": [0, 0, 0], "tail": [0, 0, 0.5]}
            ],
            "bone_meshes": {
                "root": {"profile": "circle(8)", "profile_radius": 0.1},
                "nonexistent_bone": {"profile": "circle(8)", "profile_radius": 0.1}
            }
        }
    }
)
