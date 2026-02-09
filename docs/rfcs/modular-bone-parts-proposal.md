# Modular Bone Parts: CSG-Style Mesh Composition for Bone Meshes

## Context

SpecCade's `armature_driven_v1` system builds character meshes via cumulative extrusion steps along bone axes. LLMs fail at authoring these because steps accumulate, continuous parameter optimization requires visual feedback, and there's a steep quality cliff between "plain cylinder" and "hand-tuned extrusions."

The existing proposal (`docs/rfcs/shape-templates-proposal.md`) introduces shape templates — named shapes with semantic params that compile to extrusion steps. While useful, it's still opinionated (we define what a "limb" or "torso" shape means) and limited to a fixed catalog.

**This proposal** introduces **Modular Bone Parts** — a new bone mesh creation mode where each bone's mesh is built by composing primitive shapes, referenced assets, or inline-defined static meshes using boolean operations (union, difference, intersection). This is the Three.js/CSG approach that LLMs already excel at, applied to SpecCade's rigged skeleton system.

### Why This Approach

1. **LLMs already know how to do this** — every Three.js/OpenSCAD LLM demo uses primitive composition
2. **No accumulation** — each shape is independently parameterized
3. **Not content-limited** — doesn't require us to be opinionated about anatomy
4. **Extends past bone boundaries freely** — shapes aren't constrained to [0, 1] along the bone
5. **Reuses existing infrastructure** — attachment primitives, boolean_kit patterns, asset references
6. **Coexists with extrusion steps** — use parts for segmented/kitbash characters, extrusion for organic/seamless ones

---

## Design

### New Field on `ArmatureDrivenBoneMesh`

Add a `part` field, mutually exclusive with `extrusion_steps`:

```python
# Simple: single primitive
"upper_arm_l": {
    "part": {
        "base": {"primitive": "cylinder", "dimensions": [0.12, 0.12, 1.0]},
    },
}

# Composite: base + boolean operations
"chest": {
    "part": {
        "base": {"primitive": "cylinder", "dimensions": [0.28, 0.22, 1.0]},
        "operations": [
            {"op": "union", "target": {"primitive": "sphere", "dimensions": [0.16, 0.13, 0.11], "offset": [0.11, 0.16, 0.6]}},
            {"op": "union", "target": {"primitive": "sphere", "dimensions": [0.16, 0.13, 0.11], "offset": [-0.11, 0.16, 0.6]}},
            {"op": "difference", "target": {"primitive": "cube", "dimensions": [0.5, 0.5, 0.1], "offset": [0.0, 0.0, -0.05]}},
        ],
    },
}

# Reference to an external mesh asset
"chest_armor": {
    "part": {
        "base": {"asset": "./parts/chest_plate.glb", "scale": 1.0},
    },
}

# Reference to another spec's output by asset_id
"helmet": {
    "part": {
        "base": {"asset_ref": "helmet_mesh_01"},
    },
}
```

### Coordinate Convention: The Kitbashed Skeleton

Each bone part is authored in a **normalized bone-local coordinate space**:

```
        Z = 1.0 (bone tail)
         ▲
         │  ← Part geometry lives here
         │
         │
Origin ──┼──► X      (bone head = 0,0,0)
        /
       Y
```

- **Origin (0, 0, 0)** = bone head position
- **Z axis** = bone direction. Z = 0.0 is bone head, Z = 1.0 is bone tail
- **X/Y axes** = perpendicular to bone, following bone's orientation
- Parts **CAN extend past boundaries**: Z < 0.0 (behind bone head) or Z > 1.0 (past bone tail) — no artificial constraints
- Offsets follow the same space, transformed by `_bone_local_head_to_segment_world()`

This means every bone in the skeleton is a "slot" where a modular part snaps in — origin at the bone head, stretching toward the tail. A character is a collection of parts plugged into bone slots.

### Scale Rules: Axis Mask + Z-Coupled Amount

When a part is placed on a bone, it needs to be scaled from "part space" to "world space." The `scale` field controls this with:

- `axes`: which axes are allowed to respond to bone length (`"x"`, `"y"`, `"z"`). This is effectively an axis bitmask.
- `amount_from_z`: per-axis amount in `[0.0, 1.0]` controlling how strongly each enabled axis follows total Z scale (`bone_length`).

Resolution rules:

- If `scale` is omitted: use uniform defaults (`x/y/z` enabled, all amounts `1.0`).
- If `scale: {}` or `scale` exists with `axes` omitted: also use uniform defaults.
- If `axes: []` is explicitly provided: fixed sizing (no axis follows bone length).
- If `amount_from_z` omits an enabled axis: that axis defaults to `1.0`.

Formula (after resolution):

```
factor(axis) =
  1.0                                  if axis not in axes
  1.0 + amount_from_z[axis] * (bone_length - 1.0)   otherwise

dimensions [x, y, z] -> world [x * factor(x), y * factor(y), z * factor(z)]
```

Defaults (for omitted `scale`, `scale: {}`, or missing `axes`):

```python
"scale": {
    "axes": ["x", "y", "z"],
    "amount_from_z": {"x": 1.0, "y": 1.0, "z": 1.0},
}
```

This matches prior "uniform" behavior.

Useful presets:

- Prior "uniform": `axes=["x","y","z"]`, all amounts `1.0`
- Prior "axis_only": `axes=["z"]`, `z=1.0` (X/Y fixed)
- Prior "fixed": `axes=[]` (or all amounts `0.0`)

Scale composition order:

1. Resolve axis factors from `part.scale` using `bone_length`.
2. Apply shape-local scale (`shape.scale`) if the source is `asset` or `asset_ref` (uniform scalar, default `1.0`).
3. Final world scale is per-axis multiplication of both:
   - `world_scale_x = part_factor_x * shape_scale`
   - `world_scale_y = part_factor_y * shape_scale`
   - `world_scale_z = part_factor_z * shape_scale`

For primitives, only `part.scale` applies (primitives do not have `shape.scale`).

```python
# Kitbash mode: cross-section stays fixed, length follows bone
"upper_arm_l": {
    "part": {
        "base": {"primitive": "cylinder", "dimensions": [0.08, 0.08, 1.0]},
        "scale": {"axes": ["z"], "amount_from_z": {"z": 1.0}},
    },
}

# Default mode: everything follows bone length
"upper_arm_l": {
    "part": {
        "base": {"primitive": "cylinder", "dimensions": [0.08, 0.08, 1.0]},
        # scale defaults to x/y/z all at amount 1.0
    },
}

# Hybrid mode: width/depth partially follow bone length, length fully follows
"upper_arm_l": {
    "part": {
        "base": {"primitive": "cylinder", "dimensions": [0.08, 0.08, 1.0]},
        "scale": {
            "axes": ["x", "y", "z"],
            "amount_from_z": {"x": 0.35, "y": 0.35, "z": 1.0},
        },
    },
}

# Fixed mode: exact size regardless of bone length
"helmet": {
    "part": {
        "base": {"asset": "./parts/helmet.glb"},
        "scale": {"axes": []},
    },
}
```

### Compatibility with Existing Features

When a bone mesh uses `part`, the following still work:
- **`attachments`**: Joined into the part-composed mesh (same as with extrusion meshes)
- **`modifiers`** (bevel, subdivide, bool): Applied after part composition
- **`material_index`**: Applied to all faces of the composed mesh
- **`translate` / `rotate`**: Applied to the composed mesh as a whole
- **`cap_start` / `cap_end`**: Not applicable (no extrusion caps), silently ignored
- **`connect_start` / `connect_end`**: Not applicable (no matching edge loops for bridging), validation warning if set
- **`mirror`**: Works — mirrors the resolved part-based mesh

---

## Rust Types

### New types in `crates/speccade-spec/src/recipe/character/armature_driven.rs`

Uses `#[serde(untagged)]` on `BonePartShape` — same pattern as existing `ArmatureDrivenAttachment` (disambiguated by unique key: `primitive` vs `asset` vs `asset_ref`). Inner structs get `#[serde(deny_unknown_fields)]`.

```rust
/// A bone mesh defined by composing shapes via boolean operations.
/// Alternative to extrusion_steps.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BonePart {
    /// Base shape.
    pub base: BonePartShape,

    /// Sequential boolean operations applied to the base.
    /// Applied in order: result_0 = base, result_n = op(result_{n-1}, target_n).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub operations: Vec<BonePartOperation>,

    /// How dimensions scale when placed on a bone.
    /// Default: axes x/y/z with amount 1.0 on each axis.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scale: Option<BonePartScale>,
}

/// Controls how a bone part's dimensions map to world units.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BonePartScale {
    /// Axes allowed to follow bone length.
    /// Treated as an axis bitmask internally (X=1, Y=2, Z=4).
    /// None means "use defaults" (x/y/z enabled). Some([]) means fixed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub axes: Option<Vec<BonePartScaleAxis>>,

    /// Per-axis interpolation amount from fixed (0.0) to full bone-length scaling (1.0).
    /// Any omitted axis defaults to 1.0 when the axis is enabled.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub amount_from_z: Option<BonePartScaleAmountFromZ>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BonePartScaleAxis {
    X,
    Y,
    Z,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BonePartScaleAmountFromZ {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub x: Option<f64>, // 0.0..=1.0
    #[serde(skip_serializing_if = "Option::is_none")]
    pub y: Option<f64>, // 0.0..=1.0
    #[serde(skip_serializing_if = "Option::is_none")]
    pub z: Option<f64>, // 0.0..=1.0
}

/// A shape source for bone part composition.
/// Disambiguated by unique key: `primitive`, `asset`, or `asset_ref`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum BonePartShape {
    /// Inline primitive: {"primitive": "cylinder", "dimensions": [...]}
    Primitive(BonePartPrimitive),
    /// File/path reference: {"asset": "./parts/chest.glb"}
    Asset(BonePartAsset),
    /// Asset ID reference: {"asset_ref": "my_mesh_spec_id"}
    AssetRef(BonePartAssetRef),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BonePartPrimitive {
    pub primitive: MeshPrimitive,
    /// Dimensions in bone-relative units [x, y, z].
    pub dimensions: [f64; 3],
    /// Offset from bone head in bone-local coords (bone-relative).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<[f64; 3]>,
    /// Rotation in degrees [rx, ry, rz].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rotation: Option<[f64; 3]>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BonePartAsset {
    pub asset: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<[f64; 3]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rotation: Option<[f64; 3]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Shape-local uniform scalar (applied before part-scale factors).
    pub scale: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BonePartAssetRef {
    pub asset_ref: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<[f64; 3]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rotation: Option<[f64; 3]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Shape-local uniform scalar (applied before part-scale factors).
    pub scale: Option<f64>,
}

/// A boolean operation in a bone part composition.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BonePartOperation {
    /// Boolean operation type.
    pub op: BonePartOpType,
    /// Target shape for the operation.
    pub target: BonePartShape,
}

/// Boolean operation type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BonePartOpType {
    Union,
    Difference,
    Intersect,
}
```

### Modified `ArmatureDrivenBoneMesh`

```rust
pub struct ArmatureDrivenBoneMesh {
    // ... existing fields ...

    /// Composed shape definition. Alternative to extrusion_steps.
    /// When set, the bone's mesh is built by composing shapes via CSG.
    /// Mutually exclusive with extrusion_steps.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub part: Option<BonePart>,

    // ... existing fields (extrusion_steps, attachments, modifiers, etc.) ...
}
```

### Validation Rules

Add to `crates/speccade-spec/src/validation/recipe_outputs.rs`:

1. `part` and `extrusion_steps` are mutually exclusive (error if both non-empty)
2. All `dimensions` values must be > 0
3. `scale.axes` (if present) must contain only `x|y|z` and no duplicates (error)
4. `scale.amount_from_z.*` must be within `[0.0, 1.0]` (error; no clamping)
5. If `scale` is missing or `scale.axes` is missing, normalize to uniform defaults (`x/y/z`, amounts `1.0`)
6. `connect_start` / `connect_end` = `"bridge"` is incompatible with `part` (warn)
7. `cap_start` / `cap_end` are silently ignored when `part` is set
8. Asset references resolve to existing files or known asset_ids
9. `asset.scale` / `asset_ref.scale` (if present) must be > 0

---

## Blender Python Changes

### File: `blender/speccade/armature_driven.py`

**New function**: `_build_mesh_from_part()` — parallel to existing `_build_mesh_with_steps()`

The main per-bone processing loop (around line 1509) currently calls `_build_mesh_with_steps()`. Add a branch:

```python
if 'part' in mesh_spec and mesh_spec['part']:
    obj = _build_mesh_from_part(
        part_spec=mesh_spec['part'],
        bone_name=bone_name,
        bone_length=length,
        head_w=head_world,
        tail_w=tail_world,
        base_q=base_q,
    )
else:
    obj = _build_mesh_with_steps(...)  # existing path, unchanged
```

**`_build_mesh_from_part()` implementation sketch**:

```python
def _build_mesh_from_part(*, part_spec, bone_name, bone_length, head_w, tail_w, base_q):
    """Build bone mesh from CSG composition of shapes."""

    scale_spec = part_spec.get('scale', {
        'axes': ['x', 'y', 'z'],
        'amount_from_z': {'x': 1.0, 'y': 1.0, 'z': 1.0},
    })

    # 1. Create base shape
    base_spec = part_spec['base']
    obj = _create_part_shape(
        shape_spec=base_spec,
        bone_name=bone_name,
        bone_length=bone_length,
        head_w=head_w,
        base_q=base_q,
        scale_spec=scale_spec,
        label="base",
    )

    # 2. Apply boolean operations sequentially
    for oi, op_spec in enumerate(part_spec.get('operations', [])):
        op_type = op_spec['op']  # "union", "difference", "intersect"
        target_spec = op_spec['target']

        # Create target shape (inherits same scale rules)
        target_obj = _create_part_shape(
            shape_spec=target_spec,
            bone_name=bone_name,
            bone_length=bone_length,
            head_w=head_w,
            base_q=base_q,
            scale_spec=scale_spec,
            label=f"op_{oi}",
        )

        # Apply boolean modifier
        mod = obj.modifiers.new(name=f"Bool_{oi}", type='BOOLEAN')
        mod.solver = 'EXACT'
        mod.operation = op_type.upper()
        mod.object = target_obj

        # Apply modifier immediately (deterministic)
        bpy.context.view_layer.objects.active = obj
        bpy.ops.object.modifier_apply(modifier=mod.name)

        # Remove target (it's been consumed by the boolean)
        bpy.data.objects.remove(target_obj, do_unlink=True)

    return obj


def _create_part_shape(*, shape_spec, bone_name, bone_length, head_w, base_q, scale_spec, label):
    """Create a single shape (primitive or asset) for bone part composition."""

    if 'primitive' in shape_spec:
        # Reuse existing _create_primitive_mesh
        offset = shape_spec.get('offset', [0.0, 0.0, 0.0])
        location_w = _bone_local_head_to_segment_world(offset)
        obj = _create_primitive_mesh(primitive=shape_spec['primitive'], location_w=location_w)

        # Apply dimensions based on axis-mask scale rules
        dx, dy, dz = shape_spec['dimensions']
        axes = set(scale_spec.get('axes', ['x', 'y', 'z']))
        amount = scale_spec.get('amount_from_z', {})

        def _amount_for(axis):
            v = amount.get(axis, 1.0)
            # Validation enforces [0.0, 1.0]; no runtime clamping.
            return float(v)

        def _factor(axis):
            if axis not in axes:
                return 1.0
            a = _amount_for(axis)
            return 1.0 + a * (bone_length - 1.0)

        obj.scale = (dx * _factor('x'), dy * _factor('y'), dz * _factor('z'))
        _apply_scale_only(obj)

        # Apply rotation
        rot = shape_spec.get('rotation')
        if rot is not None:
            rx, ry, rz = rot
            rot_q = Euler((math.radians(rx), math.radians(ry), math.radians(rz)), 'XYZ').to_quaternion()
            obj.rotation_quaternion = _quat_mul(base_q, rot_q)
        else:
            obj.rotation_quaternion = base_q
        _apply_rotation_only(obj)

        return obj

    elif 'asset' in shape_spec:
        # Reuse existing asset import logic from attachment processing
        # (Import .glb, apply offset/rotation in bone-relative space)
        # Scale order:
        # 1) shape-local scalar from shape_spec.get('scale', 1.0)
        # 2) part-scale factors from scale_spec (x/y/z)
        # final obj.scale = (
        #   shape_scale * _factor('x'),
        #   shape_scale * _factor('y'),
        #   shape_scale * _factor('z'),
        # )
        ...

    elif 'asset_ref' in shape_spec:
        # Resolve asset_ref to a file path, then import
        # Same scale order as 'asset' branch.
        ...
```

**Key reuse points**:
- `_create_primitive_mesh()` (line 1233) — already creates unit-size primitives
- `_bone_local_head_to_segment_world()` — converts bone-local offset to world space
- `_apply_scale_only()` / `_apply_rotation_only()` — bake transforms into vertices
- `_join_into()` — for joining attachments after part composition
- Boolean modifier application — follows same pattern as existing bool_shapes (line 1867+)

---

## Concrete Examples

### Example 1: Robot Character (All Parts)

```python
"bone_meshes": {
    "chest": {
        "part": {
            "base": {"primitive": "cube", "dimensions": [0.30, 0.22, 1.0]},
            "operations": [
                # Vent grilles (subtracted slots)
                {"op": "difference", "target": {"primitive": "cube", "dimensions": [0.20, 0.05, 0.15], "offset": [0.0, 0.12, 0.5]}},
                # Shoulder mount spheres
                {"op": "union", "target": {"primitive": "sphere", "dimensions": [0.08, 0.08, 0.08], "offset": [0.16, 0.0, 0.8]}},
                {"op": "union", "target": {"primitive": "sphere", "dimensions": [0.08, 0.08, 0.08], "offset": [-0.16, 0.0, 0.8]}},
            ],
        },
        "modifiers": [{"bevel": {"width": 0.015, "segments": 2}}],
    },
    "upper_arm_l": {
        "part": {
            "base": {"primitive": "cylinder", "dimensions": [0.10, 0.10, 1.0]},
            "operations": [
                {"op": "union", "target": {"primitive": "sphere", "dimensions": [0.12, 0.12, 0.12], "offset": [0.0, 0.0, 0.0]}},  # shoulder joint
            ],
        },
    },
    "upper_arm_r": {"mirror": "upper_arm_l"},
    "head": {
        "part": {
            "base": {"primitive": "cube", "dimensions": [0.18, 0.18, 0.9]},
            "operations": [
                # Visor (intersected inset)
                {"op": "difference", "target": {"primitive": "cube", "dimensions": [0.16, 0.05, 0.25], "offset": [0.0, 0.10, 0.5]}},
                # Antenna
                {"op": "union", "target": {"primitive": "cylinder", "dimensions": [0.015, 0.015, 0.3], "offset": [0.06, 0.0, 0.9]}},
            ],
        },
        "modifiers": [{"bevel": {"width": 0.01, "segments": 1}}],
    },
}
```

### Example 2: Mixed Approach (Extrusion Body + Part Accessories)

```python
"bone_meshes": {
    # Organic body uses extrusion steps (existing approach)
    "chest": {
        "profile": "circle(10)",
        "profile_radius": {"absolute": 0.14},
        "extrusion_steps": [
            {"extrude": 0.3, "scale": 1.15},
            {"extrude": 0.5, "scale": 1.05},
            {"extrude": 0.2, "scale": 0.7},
        ],
        "connect_end": "bridge",
    },

    # Armor pieces use parts (kitbash approach)
    "shoulder_armor_l": {
        "part": {
            "base": {"primitive": "sphere", "dimensions": [0.18, 0.14, 0.12], "offset": [0.0, 0.0, 0.5]},
            "operations": [
                {"op": "difference", "target": {"primitive": "sphere", "dimensions": [0.14, 0.10, 0.10], "offset": [0.0, 0.0, 0.5]}},
            ],
        },
        "material_index": 1,  # metal material
    },

    # Referenced asset for complex shape
    "helmet": {
        "part": {
            "base": {"asset": "./parts/knight_helmet.glb", "scale": 1.0},
        },
        "material_index": 1,
    },
}
```

### Example 3: Kitbash Character (axis-mask scale — fixed cross-sections)

```python
# With axes=["z"]: X/Y are fixed physical size, Z scales with bone
"bone_meshes": {
    "chest":      {"part": {"base": {"primitive": "cube",     "dimensions": [0.28, 0.20, 1.0]}, "scale": {"axes": ["z"], "amount_from_z": {"z": 1.0}}}},
    "spine":      {"part": {"base": {"primitive": "cube",     "dimensions": [0.22, 0.18, 1.0]}, "scale": {"axes": ["z"], "amount_from_z": {"z": 1.0}}}},
    "head":       {"part": {"base": {"primitive": "sphere",   "dimensions": [0.20, 0.20, 0.22]}, "scale": {"axes": ["z"], "amount_from_z": {"z": 1.0}}}},
    "upper_arm_l":{"part": {"base": {"primitive": "cylinder", "dimensions": [0.08, 0.08, 1.0]}, "scale": {"axes": ["z"], "amount_from_z": {"z": 1.0}}}},
    "upper_arm_r":{"mirror": "upper_arm_l"},
    "lower_arm_l":{"part": {"base": {"primitive": "cylinder", "dimensions": [0.06, 0.06, 1.0]}, "scale": {"axes": ["z"], "amount_from_z": {"z": 1.0}}}},
    "lower_arm_r":{"mirror": "lower_arm_l"},
    "hand_l":     {"part": {"base": {"primitive": "cube",     "dimensions": [0.06, 0.04, 0.7]}, "scale": {"axes": ["z"], "amount_from_z": {"z": 1.0}}}},
    "hand_r":     {"mirror": "hand_l"},
    "upper_leg_l":{"part": {"base": {"primitive": "cylinder", "dimensions": [0.10, 0.10, 1.0]}, "scale": {"axes": ["z"], "amount_from_z": {"z": 1.0}}}},
    "upper_leg_r":{"mirror": "upper_leg_l"},
    "lower_leg_l":{"part": {"base": {"primitive": "cylinder", "dimensions": [0.08, 0.08, 1.0]}, "scale": {"axes": ["z"], "amount_from_z": {"z": 1.0}}}},
    "lower_leg_r":{"mirror": "lower_leg_l"},
    "foot_l":     {"part": {"base": {"primitive": "cube",     "dimensions": [0.08, 0.12, 0.5]}, "scale": {"axes": ["z"], "amount_from_z": {"z": 1.0}}}},
    "foot_r":     {"mirror": "foot_l"},
}
```

With `axes=["z"]`, the arm cylinder is always 8cm diameter regardless of whether the arm bone is 25cm or 35cm — only the length stretches. This makes parts truly modular: swap to a different skeleton and the cross-sections stay correct.

An LLM can reason about every shape independently — "the chest is a 28cm × 20cm box" is concrete and predictable.

---

## Files to Modify

### Rust (spec types + validation)

| File | Change |
|------|--------|
| `crates/speccade-spec/src/recipe/character/armature_driven.rs` | Add `BonePart`, `BonePartScale`, `BonePartShape`, `BonePartOperation`, `BonePartOpType` types. Add `part: Option<BonePart>` field to `ArmatureDrivenBoneMesh`. |
| `crates/speccade-spec/src/validation/recipe_outputs.rs` | Add mutual exclusivity check (`part` vs `extrusion_steps`). Validate dimensions > 0. Validate `scale.axes` + `scale.amount_from_z` bounds as hard errors. Normalize missing `scale`/`axes` to uniform defaults. Validate `asset.scale`/`asset_ref.scale` > 0. Warn on `connect_start/end = bridge` with `part`. |

### Python (Blender handler)

| File | Change |
|------|--------|
| `blender/speccade/armature_driven.py` | Add `_build_mesh_from_part()` function. Add branch in per-bone processing loop to call it when `part` is present. Reuse `_create_primitive_mesh()`, `_bone_local_head_to_segment_world()`, existing asset import logic, existing boolean modifier pattern. |

### Existing code to reuse

| Function/Pattern | Location | Reuse For |
|-----------------|----------|-----------|
| `_create_primitive_mesh()` | `armature_driven.py:1233` | Creating unit-size primitives |
| `_bone_local_head_to_segment_world()` | `armature_driven.py` | Offset → world transform |
| `_apply_scale_only()` / `_apply_rotation_only()` | `armature_driven.py` | Baking transforms |
| `_join_into()` | `armature_driven.py` | Joining attachments after part composition |
| Boolean modifier pattern | `armature_driven.py:1867+` | Applying boolean ops |
| Asset import pattern | `armature_driven.py:1756+` | Importing .glb references |
| `MeshPrimitive` enum | `recipe/mesh/primitives.rs` | Primitive type validation |
| `BooleanOperation` pattern | `recipe/mesh/boolean_kit.rs` | Design reference for boolean op types |
| `MeshSource` / `MeshReference` | `recipe/mesh/boolean_kit.rs` | Design reference for shape source types |

### Test specs

| File | Purpose |
|------|---------|
| `specs/character/part_robot.star` (new) | Full robot character using only parts |
| `specs/character/part_simple.star` (new) | Simple character with one primitive per bone |
| `specs/character/part_mixed.star` (new) | Character mixing extrusion (body) + parts (accessories) |
| `specs/character/part_boolean.star` (new) | Exercises union, difference, intersection operations |
| `specs/character/part_scale_rules.star` (new) | Exercises scale defaults + explicit fixed + z-only + hybrid partial XY |
| `specs/character/part_scale_assets.star` (new) | Verifies asset/asset_ref local `scale` composes correctly with `part.scale` |

---

## Implementation Steps

1. **Write this proposal to repo** as `docs/rfcs/modular-bone-parts-proposal.md`
2. Add Rust types (`BonePart`, `BonePartShape`, etc.) to `armature_driven.rs`
3. Add `part` field to `ArmatureDrivenBoneMesh`
4. Add validation rules to `recipe_outputs.rs`
5. Implement `_build_mesh_from_part()` in `armature_driven.py`
6. Create test specs (`part_robot.star`, `part_simple.star`, `part_mixed.star`, `part_boolean.star`, `part_scale_rules.star`, `part_scale_assets.star`)
7. Run verification suite

## Verification

1. **Rust**: `cargo fmt --all && cargo clippy --workspace --all-features && cargo test --workspace`
2. **Validate specs**: `cargo run -p speccade-cli -- validate --spec specs/character/part_robot.star`
3. **Generate meshes**: `cargo run -p speccade-cli -- generate --spec specs/character/part_robot.star --out-root ./out`
4. **Visual inspection**: Open generated `.glb` files in Blender or glTF viewer
5. **Mixed mode**: Generate `part_mixed.star` to verify extrusion and parts coexist on the same character
6. **Boolean ops**: Generate `part_boolean.star` to verify union/difference/intersection produce correct geometry
7. **Mirror**: Verify `{"mirror": "upper_arm_l"}` correctly mirrors a part-based bone mesh
8. **Scale rules**: Generate `part_scale_rules.star` to verify omitted `scale` and `scale: {}` are uniform, `scale: {"axes": []}` is fixed, `scale: {"axes": ["z"], "amount_from_z": {"z": 1.0}}` is z-only, and hybrid partial XY amounts produce intermediate widths.
9. **Asset scale composition**: Generate `part_scale_assets.star` to verify `asset.scale * part.scale_factor(axis)` multiplication order
