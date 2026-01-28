# Skeletal Mesh System Rework

**Date:** 2026-01-29
**Status:** Design approved

## Problem

The current `skeletal_mesh.blender_rigged_mesh_v1` recipe is a hybrid that does neither job well:

- `body_parts` attaches primitives to bones but uses `auto_weights` (soft skinning)
- `parts` supports extrusions but spans multiple bones with soft skinning
- Neither produces clean "rigid skinned" output (1 bone per mesh part)
- Soft skinning requires precise weight control that LLMs cannot realistically provide

The legacy pure-Python system supported true armature-driven modeling (extrude geometry along bone axes), but this was lost in the Rust+Starlark migration.

## Solution

Delete `blender_rigged_mesh_v1` entirely. Replace with two focused recipe kinds:

### 1. `skeletal_mesh.armature_driven_v1`

Build mesh FROM the skeleton. Each bone defines geometry extruded along its axis. Rigid skinning only (each mesh segment 100% weighted to its bone).

**Use case:** Procedural characters, action-figure aesthetic, LLM-authorable specs.

### 2. `skeletal_mesh.skinned_mesh_v1`

Bind an existing mesh TO a skeleton. Supports both rigid binding (vertex groups) and soft skinning (auto weights).

**Use case:** Hand-authored meshes, external assets, static mesh → animatable conversion.

---

## Recipe: `skeletal_mesh.armature_driven_v1`

### Core Concept

The skeleton is the modeling guide. For each bone, geometry extrudes from head to tail. All dimensional values are **bone-relative** (fractions of bone length).

### Parameters

```yaml
recipe:
  kind: "skeletal_mesh.armature_driven_v1"
  params:
    # Skeleton definition (one of these)
    skeleton_preset: "humanoid_basic_v1"
    skeleton:  # OR custom bones
      - bone: "root"
        head: [0, 0, 0]
        tail: [0, 0, 0.1]
      - bone: "spine"
        head: [0, 0, 0.1]
        tail: [0, 0, 0.4]
        parent: "root"
      # ...

    # Per-bone mesh definitions
    bone_meshes:
      spine:
        # Profile (cross-section shape)
        profile: "hexagon(8)"           # circle(N), hexagon(N), square, rectangle
        profile_radius: 0.15            # bone-relative (15% of bone length)
        # OR elliptical:
        # profile_radius: [0.15, 0.12]  # [width, depth]
        # OR absolute escape hatch:
        # profile_radius: { absolute: 0.05 }

        # Taper (end radius as multiplier of start)
        taper: 0.9

        # Transforms
        translate: [0, 0, 0]            # bone-relative offset (negative allowed)
        rotate: [0, 0, 0]               # degrees, applied to profile before extrusion

        # Deformations
        bulge:                          # control points along bone axis (0.0 = head, 1.0 = tail)
          - at: 0.0
            scale: 0.8
          - at: 0.5
            scale: 1.2
          - at: 1.0
            scale: 0.9
        twist: 0                        # degrees rotation along bone axis

        # Caps
        cap_start: true
        cap_end: false                  # false if connecting to child bone

        # Modifiers (applied in order)
        modifiers:
          - bevel: { width: 0.02, segments: 2 }
          - subdivide: { cuts: 1 }
          - bool: { operation: "subtract", target: "eye_socket_L" }

        material_index: 0

        # Attachments (optional, for geometry not on bone axis)
        attachments:
          # Type A: Primitives
          - primitive: "sphere"
            dimensions: [0.08, 0.06, 0.08]  # bone-relative
            offset: [0.15, 0.05, 0.3]
            rotation: [0, 0, 15]
            material_index: 1

          # Type B: Mini extrusion (explicit start/end, not following bone)
          - extrude:
              profile: "hexagon(4)"
              start: [0, 0.1, 0.5]          # local to bone
              end: [0, 0.2, 0.6]
              profile_radius: 0.05
              taper: 0.3

          # Type C: Asset reference
          - asset: "props/shoulder_armor.glb"
            offset: [0.18, 0, 0.4]
            rotation: [0, 0, 0]
            scale: 1.0

      # Mirror syntax for symmetric bones
      arm_upper_L:
        profile: "hexagon(6)"
        profile_radius: 0.12
        taper: 0.85
        # ...
      arm_upper_R:
        mirror: "arm_upper_L"

    # Bool shapes (for subtraction, not rendered)
    bool_shapes:
      eye_socket_L:
        primitive: "sphere"
        dimensions: [0.06, 0.08, 0.06]
        position: [0.05, 0.15, 0.6]       # bone-relative to associated bone
      eye_socket_R:
        mirror: "eye_socket_L"

    # Materials
    material_slots:
      - name: "skin"
        base_color: [0.85, 0.7, 0.55, 1.0]
        metallic: 0.0
        roughness: 0.6
      - name: "armor"
        base_color: [0.3, 0.3, 0.35, 1.0]
        metallic: 0.8
        roughness: 0.3

    # Export settings
    export:
      include_armature: true
      include_normals: true
      include_uvs: true
      triangulate: true
      include_skin_weights: true

    # Constraints
    constraints:
      max_triangles: 3000
      max_bones: 30
      max_materials: 4
```

### Bone-Relative Units

All dimensional values are fractions of bone length (head → tail distance):

- `profile_radius: 0.15` → radius is 15% of bone length
- `translate: [0.1, 0, 0]` → shift 10% of bone length in X
- `bulge.at: 0.5` → midpoint of bone

**Benefit:** Resize skeleton → mesh scales proportionally. Presets with different scales just work.

**Escape hatch:** `{ absolute: 0.05 }` for exact world units when needed.

### Bones Without Geometry

Bones not listed in `bone_meshes` get no geometry. Useful for:
- IK targets
- Helper bones
- Animation-only bones

---

## Recipe: `skeletal_mesh.skinned_mesh_v1`

### Core Concept

Bind an existing mesh to a skeleton. The mesh can come from a file, another speccade asset, or any external source.

### Parameters

```yaml
recipe:
  kind: "skeletal_mesh.skinned_mesh_v1"
  params:
    # Mesh source (one of these)
    mesh_file: "path/to/mesh.glb"       # external file path
    mesh_asset: "my_character_mesh"     # reference speccade asset by ID

    # Skeleton definition (same as armature_driven)
    skeleton_preset: "humanoid_basic_v1"
    # OR
    skeleton: [...]                      # custom bones

    # Binding configuration
    binding:
      mode: "rigid"                      # or "auto_weights"

      # For rigid mode: map mesh vertex groups to bones
      # If mesh vertex groups already match bone names, auto-mapped
      vertex_group_map:
        "Arm.L": "arm_upper_L"           # mesh group → bone name
        "Arm.R": "arm_upper_R"
        # unmapped groups ignored

      # For auto_weights mode
      max_bone_influences: 4

    # Optional: override materials from source mesh
    material_slots:
      - name: "skin"
        base_color: [0.85, 0.7, 0.55, 1.0]

    # Export settings
    export:
      include_armature: true
      include_normals: true
      include_uvs: true
      triangulate: true
      include_skin_weights: true

    # Constraints
    constraints:
      max_triangles: 5000
      max_bones: 64
      max_materials: 8
```

### Binding Modes

**`rigid`:** Each vertex belongs 100% to one bone. Requires vertex groups in source mesh (either matching bone names or mapped via `vertex_group_map`).

**`auto_weights`:** Blender calculates smooth skinning based on bone proximity. Good for organic meshes. Respects `max_bone_influences`.

---

## Migration

### Deleted

- `skeletal_mesh.blender_rigged_mesh_v1` - removed entirely, no deprecation period

### Golden Specs to Recreate

| Old spec | New recipe |
|----------|------------|
| `humanoid_male.spec.json` | `armature_driven_v1` |
| `humanoid_female.spec.json` | `armature_driven_v1` |
| `quadruped_dog.spec.json` | `armature_driven_v1` |
| `creature_spider.spec.json` | `armature_driven_v1` |
| `character_humanoid.star` | `armature_driven_v1` |
| `character_humanoid_blank.star` | `armature_driven_v1` |

### New Golden Specs to Add

- At least one `skinned_mesh_v1` example demonstrating mesh import + rig binding

---

## Animation Integration

Both recipes output identical GLB structure: armature + skinned mesh. The animation system (`skeletal_animation.*` recipes) works with either.

```
skeletal_mesh.armature_driven_v1  ─┐
                                   ├──► GLB (armature + mesh) ──► skeletal_animation.*
skeletal_mesh.skinned_mesh_v1    ─┘
```

Skeleton compatibility: animations targeting a skeleton preset work with any mesh rigged to that preset.

---

## Out of Scope

- LOD generation
- Blend shapes / morph targets
- Cloth simulation
- IK solving (handled by animation recipes)
