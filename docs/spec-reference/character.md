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
| `bone_meshes` | object | No | Per-bone mesh definitions (extrusion path or `part` composition path) |
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

### Modular Bone Parts (`bone_meshes.<bone>.part`)

`part` is an alternative to `extrusion_steps`. It builds a bone mesh by composing shapes:

- `base`: required shape (`primitive`, `asset`, or `asset_ref`)
- `operations[]`: optional ordered boolean ops (`union`, `difference`, `intersect`)
- `scale`: optional axis-mask scaling policy coupled to bone length

Rules:

- `part` and `extrusion_steps` are mutually exclusive.
- `attachments`, `modifiers`, `material_index`, `translate`, and `rotate` still apply.
- `cap_start` / `cap_end` are ignored when `part` is present.
- `connect_start` / `connect_end = "bridge"` is ignored for `part` meshes (validation warning).

Scale defaults:

- Omitted `scale` (or `scale: {}`) behaves as uniform follow: axes `x/y/z` with amount `1.0` each.
- `scale.axes = []` means fixed size (no axis follows bone length).
- If an enabled axis omits `amount_from_z.<axis>`, it defaults to `1.0`.

### Units + Bone-Relative Semantics

- Most per-bone geometry fields are specified in *bone-local* coordinates (origin at the bone head).
- Bone-local `z` is the **bone axis** (`head -> tail`); `x/y` are perpendicular and rotate with the bone.
- *Bone-relative units* mean values are interpreted as a fraction of the bone length (head-to-tail distance).
- Angles are in degrees.

Axis caution (common foot-direction pitfall):
- For bones that point forward (for example `foot_*`), forward extension is typically local `+z`, **not** local `+y`.

Common bone-relative fields:

- `bone_meshes.<bone>.translate`: `[x, y, z]` offset in bone-local, bone-relative units.
- `bone_meshes.<bone>.rotate`: `[x, y, z]` profile rotation in degrees (applied before extrusion).
- `bone_meshes.<bone>.profile_radius`:
  - number: uniform radius in bone-relative units
  - `[x, y]`: elliptical radius in bone-relative units
  - `{ "absolute": a }`: absolute units escape hatch (not scaled by bone length)
- `bone_meshes.<bone>.bulge[*].at`: normalized position along the bone axis (`0.0` = head, `1.0` = tail)
- `bone_meshes.<bone>.twist`: degrees of twist along the bone axis

Attachment/bool-shape fields are also bone-relative:

- `bone_meshes.<bone>.attachments[*].dimensions` / `.offset`: bone-relative units
- `bone_meshes.<bone>.attachments[*].extrude.start` / `.end`: bone-local, bone-relative units
- `bool_shapes.<shape>.position` / `.dimensions`: bone-local, bone-relative units (when `bone` is set)

### Boolean Operations

- Boolean modifiers reference `bool_shapes` by name.
- Supported boolean operations: `difference`/`subtract`, `union`, `intersect`/`intersection`.
- `bool_shapes` are helper shapes and are not exported in the final GLB.

### Mirror References

- `bone_meshes` and `bool_shapes` support mirror references: `{ "mirror": "other_key" }`.
- Mirrors are a *copy-by-reference* convenience (the resolved definition is reused as-is).
- Any visual left/right mirroring comes from using mirrored bones in the skeleton (or custom skeleton bones with `mirror`).

### Bridge Edge Loops

Connect bone mesh segments topologically using bridge edge loops for smooth deformation at joints.

**Connection modes:**
- `"segmented"` (default): Mesh ends are independent, capped or uncapped per `cap_start`/`cap_end`
- `"bridge"`: Merge edge loops with adjacent bone's mesh, blend weights at junction

**Per-bone mesh fields:**

| Field | Type | Default | Notes |
|-------|------|---------|-------|
| `connect_start` | string | `"segmented"` | How this bone's mesh start connects to parent bone's mesh end |
| `connect_end` | string | `"segmented"` | How this bone's mesh end connects to child bones' mesh starts |

**Requirements for bridging:**
- Both parent's `connect_end` and child's `connect_start` must be `"bridge"`
- Profile segment counts must be compatible (exact match or 2x multiple)
- Both bones must have mesh definitions (not mirror references)

**Example - connected torso:**

```json
{
  "bone_meshes": {
    "spine": {
      "profile": "circle(8)",
      "connect_end": "bridge"
    },
    "chest": {
      "profile": "circle(8)",
      "connect_start": "bridge",
      "connect_end": "bridge"
    },
    "neck": {
      "profile": "circle(8)",
      "connect_start": "bridge"
    }
  }
}
```

**Weight blending:** Vertices in the bridge region receive interpolated weights between the parent and child bones based on their position along the bridge axis.

### Export Settings

- `params.export` is honored by the Blender backend; it affects GLB export (triangulation, normals, UVs, skin weights, and whether the armature is included).

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
