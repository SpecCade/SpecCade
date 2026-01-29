# Character (Skeletal Mesh) Specs

Skeletal meshes are rigged 3D models (armature + mesh) exported as GLB.

| Property | Value |
|----------|-------|
| Asset Type | `skeletal_mesh` |
| Recipe Kinds | `skeletal_mesh.armature_driven_v1`, `skeletal_mesh.skinned_mesh_v1` |
| Output Formats | `glb` |
| Determinism | Tier 2 (metric validation) |

> **Coordinate System:** SpecCade uses Z-up, Y-forward convention (Blender standard). See [Coordinate System Conventions](../conventions/coordinate-system.md) for details.
> - Character origin at feet (Z=0)
> - Character faces +Y in rest pose
> - Left side at -X, right side at +X

## SSOT (Source Of Truth)

- Rust types: `crates/speccade-spec/src/recipe/character/`
- Starlark specs: `specs/character/`
- CLI validation: `speccade validate --spec file.json`

## Recipe: `skeletal_mesh.armature_driven_v1`

Build mesh **from** the skeleton (armature-driven modeling). Output uses rigid skinning (generated segments are 100% weighted to their bone).

### Params (Concise)

| Field | Type | Required | Notes |
|-------|------|----------|-------|
| `skeleton_preset` | string | No* | Preset skeleton ID (e.g. `humanoid_basic_v1`) |
| `skeleton` | array | No* | Custom bones; at least one of `skeleton_preset` / `skeleton` |
| `bone_meshes` | object | No | Per-bone mesh definitions (profile/extrusion/modifiers) |
| `bool_shapes` | object | No | Named boolean subtraction shapes referenced by modifiers |
| `material_slots` | array | No | Material slots (PBR-ish params) |
| `export` | object | No | GLB export settings (triangulation, normals, uvs, weights) |
| `constraints` | object | No | Metric constraints (triangles, bones, materials) |

Minimal recipe example:

```json
{
  "recipe": {
    "kind": "skeletal_mesh.armature_driven_v1",
    "params": {
      "skeleton_preset": "humanoid_basic_v1",
      "bone_meshes": {
        "spine": { "profile": "hexagon(8)", "profile_radius": 0.15, "taper": 0.9 }
      }
    }
  }
}
```

## Recipe: `skeletal_mesh.skinned_mesh_v1`

Bind an existing mesh **to** a skeleton. Supports rigid binding (vertex groups) and soft skinning (auto weights).

### Params (Concise)

| Field | Type | Required | Notes |
|-------|------|----------|-------|
| `mesh_file` | string | No* | External mesh file path (e.g. `*.glb`) |
| `mesh_asset` | string | No* | Reference another SpecCade asset as the mesh source |
| `skeleton_preset` | string | No* | Preset skeleton ID |
| `skeleton` | array | No* | Custom bones |
| `binding` | object | No | Binding config (`mode`, optional `vertex_group_map`, `max_bone_influences`) |
| `material_slots` | array | No | Optional material overrides |
| `export` | object | No | GLB export settings |
| `constraints` | object | No | Metric constraints |

*At least one of `mesh_file` / `mesh_asset` and at least one of `skeleton_preset` / `skeleton` should be provided.

Minimal recipe example:

```json
{
  "recipe": {
    "kind": "skeletal_mesh.skinned_mesh_v1",
    "params": {
      "mesh_file": "assets/character.glb",
      "skeleton_preset": "humanoid_basic_v1",
      "binding": { "mode": "rigid" }
    }
  }
}
```

## See Also

- [Static Mesh Specs](mesh.md) - Non-deforming meshes
- [Animation Specs](animation.md) - Skeletal animations
