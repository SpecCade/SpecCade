# Character / Humanoid Reference

Quick reference for LLM-authored humanoid characters using the SpecCade Starlark stdlib.

## `humanoid_basic_v1` Bone List

| Bone | Typical Primitive | Notes |
|------|-------------------|-------|
| `spine` | cylinder | Lower torso |
| `chest` | cylinder | Upper torso |
| `head` | sphere | Head |
| `upper_arm_l` | cylinder | Left upper arm |
| `upper_arm_r` | cylinder | Right upper arm |
| `upper_leg_l` | cylinder | Left upper leg |
| `upper_leg_r` | cylinder | Right upper leg |

Additional bones (`lower_arm_*`, `hand_*`, `lower_leg_*`, `foot_*`, `neck`) are available but optional for simple placeholders.

## Blank Material Convention

Use a single white material when the final look comes from textures or runtime shading:

```starlark
material_slots = [
    material_slot(
        name = "blank_white",
        base_color = [1.0, 1.0, 1.0, 1.0]
    ),
]
```

All `body_part()` calls use `material_index = 0`.

## Starlark Template

```starlark
skeletal_mesh_spec(
    asset_id = "my-humanoid-01",
    seed = 42,
    output_path = "characters/my_humanoid.glb",
    format = "glb",
    skeleton_preset = "humanoid_basic_v1",
    description = "Blank humanoid placeholder",
    body_parts = [
        body_part(bone = "spine",       primitive = "cylinder", dimensions = [0.25, 0.4, 0.25],  segments = 8,  offset = [0,0,0.3],  material_index = 0),
        body_part(bone = "chest",       primitive = "cylinder", dimensions = [0.3, 0.3, 0.28],   segments = 8,  offset = [0,0,0.6],  material_index = 0),
        body_part(bone = "head",        primitive = "sphere",   dimensions = [0.15, 0.18, 0.15], segments = 12, offset = [0,0,0.95], material_index = 0),
        body_part(bone = "upper_arm_l", primitive = "cylinder", dimensions = [0.06, 0.25, 0.06], segments = 6,  rotation = [0,0,90],  material_index = 0),
        body_part(bone = "upper_arm_r", primitive = "cylinder", dimensions = [0.06, 0.25, 0.06], segments = 6,  rotation = [0,0,-90], material_index = 0),
        body_part(bone = "upper_leg_l", primitive = "cylinder", dimensions = [0.08, 0.35, 0.08], segments = 6,  rotation = [180,0,0], material_index = 0),
        body_part(bone = "upper_leg_r", primitive = "cylinder", dimensions = [0.08, 0.35, 0.08], segments = 6,  rotation = [180,0,0], material_index = 0),
    ],
    material_slots = [
        material_slot(name = "blank_white", base_color = [1.0, 1.0, 1.0, 1.0]),
    ],
    skinning = skinning_config(max_bone_influences = 4, auto_weights = True),
    export = skeletal_export_settings(triangulate = True, include_skin_weights = True),
    constraints = skeletal_constraints(max_triangles = 5000, max_bones = 64, max_materials = 4),
    texturing = skeletal_texturing(uv_mode = "cylinder_project")
)
```

## See Also

- [Starlark Authoring Guide](../../../../docs/starlark-authoring.md)
- [Character Spec Reference](../../../../docs/spec-reference/character.md)
- [Golden example](../../../../golden/starlark/character_humanoid_blank.star)
