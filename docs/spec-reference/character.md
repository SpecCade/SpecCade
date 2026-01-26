# Character (Skeletal Mesh) Specs

Skeletal meshes are rigged 3D models with armatures for characters, creatures, and animated objects.

| Property | Value |
|----------|-------|
| Asset Type | `skeletal_mesh` |
| Recipe Kind | `skeletal_mesh.blender_rigged_mesh_v1` |
| Output Formats | `glb` |
| Determinism | Tier 2 (metric validation) |

> **Coordinate System:** SpecCade uses Z-up, Y-forward convention (Blender standard). See [Coordinate System Conventions](../conventions/coordinate-system.md) for details.
> - Character origin at feet (Z=0)
> - Character faces +Y in rest pose
> - Left side at -X, right side at +X

## SSOT (Source Of Truth)

- Rust types: `crates/speccade-spec/src/recipe/character/`
- Golden specs: `golden/speccade/specs/skeletal_mesh/`
- CLI validation: `speccade validate --spec file.json`

## Recipe Parameters

The `skeletal_mesh.blender_rigged_mesh_v1` recipe builds rigged meshes with skeletons and body parts.

### Main Parameters

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `skeleton_preset` | string | No* | Predefined skeleton rig |
| `skeleton` | array | No* | Custom skeleton definition |
| `body_parts` | array | No | Body part mesh definitions |
| `parts` | object | No | Legacy parts (dict-style, keyed by name) |
| `material_slots` | array | No | Material definitions |
| `skinning` | object | No | Skinning/weight settings |
| `export` | object | No | Export settings |
| `constraints` | object | No | Validation constraints |
| `tri_budget` | u32 | No | Triangle budget |
| `texturing` | object | No | Texturing options |

*At least one of `skeleton_preset` or `skeleton` should be provided.

### Skeleton Presets

| Value | Description | Bone Count |
|-------|-------------|------------|
| `humanoid_basic_v1` | Basic humanoid skeleton | 20 |

**Humanoid Basic V1 Bones:**
- Root: `root`, `hips`
- Spine: `spine`, `chest`, `neck`, `head`
- Left arm: `shoulder_l`, `upper_arm_l`, `lower_arm_l`, `hand_l`
- Right arm: `shoulder_r`, `upper_arm_r`, `lower_arm_r`, `hand_r`
- Left leg: `upper_leg_l`, `lower_leg_l`, `foot_l`
- Right leg: `upper_leg_r`, `lower_leg_r`, `foot_r`

### Custom Skeleton

Define custom bones when a preset doesn't fit:

```json
"skeleton": [
  {
    "bone": "root",
    "head": [0, 0, 0],
    "tail": [0, 0, 0.1]
  },
  {
    "bone": "spine",
    "parent": "root",
    "head": [0, 0, 0.1],
    "tail": [0, 0, 0.3]
  },
  {
    "bone": "arm_r",
    "parent": "spine",
    "mirror": "arm_l"
  }
]
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `bone` | string | Yes | Unique bone name |
| `head` | `[f64; 3]` | No | Head position [X (right), Y (forward), Z (up)] in meters |
| `tail` | `[f64; 3]` | No | Tail position [X (right), Y (forward), Z (up)] in meters |
| `parent` | string | No | Parent bone name |
| `mirror` | string | No | Mirror from another bone (L->R reflection) |

### Body Parts

Body parts attach mesh primitives to bones:

```json
"body_parts": [
  {
    "bone": "chest",
    "mesh": {
      "primitive": "cylinder",
      "dimensions": [0.3, 0.3, 0.28],
      "segments": 8,
      "offset": [0, 0, 0.6],
      "rotation": [0, 0, 0]
    },
    "material_index": 0
  }
]
```

#### BodyPart

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `bone` | string | Yes | Bone to attach to |
| `mesh` | object | Yes | Mesh configuration |
| `material_index` | u32 | No | Material slot index |

#### BodyPartMesh

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `primitive` | string | Yes | Primitive type: `cube`, `sphere`, `cylinder`, `cone`, `torus`, `plane`, `ico_sphere` |
| `dimensions` | `[f64; 3]` | Yes | Size [X, Y, Z] |
| `segments` | u8 | No | Subdivision segments |
| `offset` | `[f64; 3]` | No | Position offset from bone [X (right), Y (forward), Z (up)] |
| `rotation` | `[f64; 3]` | No | Euler rotation [X (pitch), Y (roll), Z (yaw)] in degrees |

### Material Slots

Same as static mesh materials:

```json
"material_slots": [
  {
    "name": "body_material",
    "base_color": [0.8, 0.6, 0.5, 1.0]
  },
  {
    "name": "head_material",
    "base_color": [0.9, 0.7, 0.6, 1.0]
  }
]
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | string | Yes | Material name |
| `base_color` | `[f64; 4]` | No | RGBA color (0.0 to 1.0) |
| `metallic` | f64 | No | Metallic value (0.0 to 1.0) |
| `roughness` | f64 | No | Roughness value (0.0 to 1.0) |
| `emissive` | `[f64; 3]` | No | Emissive RGB color |
| `emissive_strength` | f64 | No | Emissive strength |

### Skinning Settings

```json
"skinning": {
  "max_bone_influences": 4,
  "auto_weights": true
}
```

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `max_bone_influences` | u8 | 4 | Max bone influences per vertex (1-8) |
| `auto_weights` | bool | true | Use automatic weight painting |

### Export Settings

```json
"export": {
  "include_armature": true,
  "include_normals": true,
  "include_uvs": true,
  "triangulate": true,
  "include_skin_weights": true,
  "save_blend": false
}
```

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `include_armature` | bool | true | Include armature in export |
| `include_normals` | bool | true | Include vertex normals |
| `include_uvs` | bool | true | Include UV coordinates |
| `triangulate` | bool | true | Triangulate mesh |
| `include_skin_weights` | bool | true | Include skin weights |
| `save_blend` | bool | false | Save .blend file alongside GLB |

### Constraints

```json
"constraints": {
  "max_triangles": 5000,
  "max_bones": 64,
  "max_materials": 4
}
```

| Field | Type | Description |
|-------|------|-------------|
| `max_triangles` | u32 | Maximum triangle count |
| `max_bones` | u32 | Maximum bone count |
| `max_materials` | u32 | Maximum material count |

### Texturing

```json
"texturing": {
  "uv_mode": "cylinder_project"
}
```

| Field | Type | Description |
|-------|------|-------------|
| `uv_mode` | string | UV unwrapping mode |

## Example Spec

```json
{
  "spec_version": 1,
  "asset_id": "preset_humanoid",
  "asset_type": "skeletal_mesh",
  "license": "CC0-1.0",
  "seed": 7005,
  "description": "Humanoid using skeleton_preset",
  "outputs": [
    {
      "kind": "primary",
      "format": "glb",
      "path": "preset_humanoid.glb"
    }
  ],
  "recipe": {
    "kind": "skeletal_mesh.blender_rigged_mesh_v1",
    "params": {
      "skeleton_preset": "humanoid_basic_v1",
      "body_parts": [
        {
          "bone": "chest",
          "mesh": {
            "primitive": "cylinder",
            "dimensions": [0.3, 0.3, 0.28],
            "segments": 8,
            "offset": [0, 0, 0.6]
          },
          "material_index": 0
        },
        {
          "bone": "head",
          "mesh": {
            "primitive": "sphere",
            "dimensions": [0.15, 0.18, 0.15],
            "segments": 12,
            "offset": [0, 0, 0.95]
          },
          "material_index": 1
        }
      ],
      "material_slots": [
        {
          "name": "body_material",
          "base_color": [0.8, 0.6, 0.5, 1.0]
        },
        {
          "name": "head_material",
          "base_color": [0.9, 0.7, 0.6, 1.0]
        }
      ],
      "skinning": {
        "max_bone_influences": 4,
        "auto_weights": true
      },
      "export": {
        "include_armature": true,
        "triangulate": true,
        "include_skin_weights": true
      },
      "constraints": {
        "max_triangles": 5000,
        "max_bones": 64
      }
    }
  }
}
```

## Output Metrics

Generation produces a report with mesh and skeleton metrics:

| Metric | Description |
|--------|-------------|
| `vertex_count` | Number of vertices |
| `face_count` | Number of faces |
| `triangle_count` | Number of triangles |
| `bone_count` | Number of bones |
| `max_bone_influences` | Max influences per vertex |
| `unweighted_vertex_count` | Vertices with zero weight |
| `weight_normalization_percentage` | Vertices with normalized weights |
| `manifold` | Whether mesh is watertight |
| `material_slot_count` | Number of materials |
| `bounding_box` | Axis-aligned bounding box |

## Post-Generation Verification

Use `speccade verify` to validate skeletal mesh metrics:

```bash
speccade verify --report output.report.json --constraints constraints.json
```

Skeletal mesh constraints include:
- `MaxBoneCount` - Limit bone count
- `MaxBoneInfluences` - Limit influences per vertex
- `MaxUnweightedVertices` - Limit unweighted vertices
- `MinWeightNormalization` - Require weight normalization

## See Also

- [Static Mesh Specs](mesh.md) - Non-deforming meshes
- [Animation Specs](animation.md) - Skeletal animations
