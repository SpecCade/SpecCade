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
    body_parts = [
        # Torso
        body_part(
            bone = "spine",
            primitive = "cylinder",
            dimensions = [0.25, 0.4, 0.25],
            segments = 8,
            offset = [0, 0, 0.3],
            material_index = 0
        ),
        body_part(
            bone = "chest",
            primitive = "cylinder",
            dimensions = [0.3, 0.3, 0.28],
            segments = 8,
            offset = [0, 0, 0.6],
            material_index = 0
        ),
        # Head
        body_part(
            bone = "head",
            primitive = "sphere",
            dimensions = [0.15, 0.18, 0.15],
            segments = 12,
            offset = [0, 0, 0.95],
            material_index = 1
        ),
        # Arms
        body_part(
            bone = "upper_arm_l",
            primitive = "cylinder",
            dimensions = [0.06, 0.25, 0.06],
            segments = 6,
            rotation = [0, 0, 90],
            material_index = 0
        ),
        body_part(
            bone = "upper_arm_r",
            primitive = "cylinder",
            dimensions = [0.06, 0.25, 0.06],
            segments = 6,
            rotation = [0, 0, -90],
            material_index = 0
        ),
        # Legs
        body_part(
            bone = "upper_leg_l",
            primitive = "cylinder",
            dimensions = [0.08, 0.35, 0.08],
            segments = 6,
            rotation = [180, 0, 0],
            material_index = 0
        ),
        body_part(
            bone = "upper_leg_r",
            primitive = "cylinder",
            dimensions = [0.08, 0.35, 0.08],
            segments = 6,
            rotation = [180, 0, 0],
            material_index = 0
        ),
    ],
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
    skinning = skinning_config(
        max_bone_influences = 4,
        auto_weights = True
    ),
    export = skeletal_export_settings(
        triangulate = True,
        include_skin_weights = True
    ),
    constraints = skeletal_constraints(
        max_triangles = 5000,
        max_bones = 64,
        max_materials = 4
    ),
    texturing = skeletal_texturing(uv_mode = "cylinder_project")
)
