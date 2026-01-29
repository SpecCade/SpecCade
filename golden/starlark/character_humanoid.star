# Simple humanoid character - demonstrates character stdlib
#
# This example creates a basic humanoid skeletal mesh using the
# character stdlib functions for body parts, materials, and skinning.
#
# [VALIDATION]
# SHAPE: Humanoid figure with cylindrical torso, spherical head, cylindrical limbs
# PROPORTIONS: Head ~0.15 units, torso segments ~0.3-0.4 units, arms ~0.25 units, legs ~0.35 units
# ORIENTATION: Standing upright (+Z up), facing +Y forward, T-pose with arms extended sideways
# FRONT VIEW: Symmetric humanoid shape - head on top, arms extended left/right, legs below
# BACK VIEW: Mirror of front, spine area visible
# LEFT VIEW: Profile view - head, single arm cylinder, single leg cylinder
# RIGHT VIEW: Mirror of left view
# TOP VIEW: Looking down at head sphere, torso below, arms extending sideways
# ISO VIEW: Full 3D humanoid form clearly visible in T-pose
# NOTES: Body uses skin-tone material (0.8, 0.6, 0.5), head slightly lighter (0.9, 0.7, 0.6)

skeletal_mesh_spec(
    asset_id = "stdlib-character-humanoid-01",
    seed = 42,
    output_path = "characters/humanoid.glb",
    format = "glb",
    skeleton_preset = "humanoid_basic_v1",
    description = "Stdlib humanoid character example",
    bone_meshes = {
        # Torso (extruded along bone axes)
        "spine": {
            "profile": "circle(8)",
            "profile_radius": 0.15,
            "material_index": 0,
            "cap_start": True,
            "cap_end": False,
        },
        "chest": {
            "profile": "circle(8)",
            "profile_radius": 0.18,
            "material_index": 0,
            "cap_start": False,
            "cap_end": True,
        },
        # Head: small neck extrusion + spherical attachment
        "head": {
            "profile": "circle(8)",
            "profile_radius": 0.08,
            "material_index": 1,
            "cap_start": False,
            "cap_end": True,
            "attachments": [
                {
                    "primitive": "sphere",
                    "dimensions": [0.15, 0.18, 0.15],
                    "offset": [0, 0, 0.6],
                    "material_index": 1,
                },
            ],
        },
        # Arms
        "upper_arm_l": {
            "profile": "circle(6)",
            "profile_radius": 0.08,
            "material_index": 0,
            "rotate": [0, 0, 90],
        },
        "upper_arm_r": {
            "profile": "circle(6)",
            "profile_radius": 0.08,
            "material_index": 0,
            "rotate": [0, 0, -90],
        },
        # Legs
        "upper_leg_l": {
            "profile": "circle(6)",
            "profile_radius": 0.1,
            "material_index": 0,
            "rotate": [180, 0, 0],
        },
        "upper_leg_r": {
            "profile": "circle(6)",
            "profile_radius": 0.1,
            "material_index": 0,
            "rotate": [180, 0, 0],
        },
    },
    material_slots = [
        material_slot(
            name = "body_material",
            base_color = [0.8, 0.6, 0.5, 1.0]
        ),
        material_slot(
            name = "head_material",
            base_color = [0.9, 0.7, 0.6, 1.0]
        ),
    ],
    export = skeletal_export_settings(
        triangulate = True,
        include_skin_weights = True
    ),
    constraints = skeletal_constraints(
        max_triangles = 5000,
        max_bones = 64,
        max_materials = 4
    )
)
