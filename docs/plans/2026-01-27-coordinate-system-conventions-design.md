# SpecCade Coordinate System Conventions - Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Document and enforce SpecCade's coordinate conventions to improve LLM-generated mesh and animation quality.

**Architecture:** Add coordinate system documentation as the canonical reference, update spec-reference docs with convention headers, extend stdlib dump with coordinate metadata, and add semantic validation warnings for unusual orientations.

**Tech Stack:** Rust (speccade-spec, speccade-cli), Markdown documentation, JSON schema

---

## Task 1: Create Canonical Coordinate System Documentation

Create the primary reference document for SpecCade's coordinate conventions.

**Files:**
- Create: `docs/conventions/coordinate-system.md`

**Step 1: Create conventions directory**

Run: `mkdir docs/conventions` (if not exists)

**Step 2: Write the coordinate system documentation**

```markdown
# SpecCade Coordinate System Conventions

This document defines the canonical coordinate system conventions for all SpecCade mesh and animation assets. Following these conventions ensures consistent, predictable asset generation.

## Coordinate System Overview

SpecCade uses **Blender convention** (right-handed, Z-up):

```
        +Z (Up)
         |
         |
         |_______ +Y (Forward)
        /
       /
      +X (Right)
```

| Axis | Direction | Anatomical Meaning |
|------|-----------|-------------------|
| **+X** | Right | Character's right side |
| **+Y** | Forward | Direction character faces |
| **+Z** | Up | Vertical (head direction) |

**Handedness:** Right-handed (cross product: X × Y = Z)

## GLB Export Conversion

Blender's GLB exporter automatically converts to glTF convention (Y-up):
- Blender Z-up → glTF Y-up
- Blender Y-forward → glTF -Z forward

**Spec authors should always think in Blender convention.** The export handles conversion.

## Position Convention

Positions are specified as `[X, Y, Z]` in **meters** (Blender units):

| Example | Meaning |
|---------|---------|
| `[0, 0, 0]` | Origin (typically at character's feet) |
| `[0, 0, 1]` | 1 meter directly above origin |
| `[0, 1, 0]` | 1 meter forward of origin |
| `[1, 0, 0]` | 1 meter to the right of origin |
| `[-0.3, 0, 1]` | Left shoulder area (left, neutral, elevated) |

## Rotation Convention

Euler rotations are specified as `[X, Y, Z]` in **degrees**:

| Component | Axis | Motion |
|-----------|------|--------|
| `rotation[0]` | X | Pitch (nod forward/back) |
| `rotation[1]` | Y | Roll (tilt left/right) |
| `rotation[2]` | Z | Yaw (turn left/right) |

**Positive rotation:** Counter-clockwise when looking down the axis toward origin (right-hand rule).

**Euler order:** XYZ (Blender default).

## Skeleton Conventions

### Humanoid Reference Pose

The standard humanoid stands at origin in **T-pose**:
- Feet at Z=0 (ground plane)
- Facing +Y direction
- Arms extended along X axis (left arm toward -X, right arm toward +X)
- Spine aligned with +Z axis

```
      Head (Z ≈ 1.6)
        |
   L ---+--- R  (arms along X)
        |
       Hips (Z ≈ 0.9)
      /   \
     L     R  (legs)
    /       \
  Feet (Z = 0)

  Facing → +Y
```

### Bone Orientation

Bones point from `head` to `tail`:
- Spine bones: tail above head (+Z direction)
- Arm bones: tail toward hand (±X direction)
- Leg bones: tail toward foot (-Z direction)

### Naming Convention

- Left/right suffixes: `_l` / `_r`
- Hierarchy: `upper_arm_l` → `lower_arm_l` → `hand_l`

## Mesh Conventions

### Static Meshes

- Origin at geometric center or base (context-dependent)
- "Front" of object faces +Y
- "Top" of object faces +Z

### Character Meshes

- Origin at feet (ground contact point)
- Character faces +Y in rest pose
- Symmetric across YZ plane (X=0)

## Quadruped Convention

Four-legged creatures follow the same axes:
- Spine runs along Y axis (head toward +Y)
- Legs extend downward (-Z)
- Left legs at -X, right legs at +X

```
  Head → +Y
    |
  Spine (along Y)
   /||\
  L    R  (legs down toward -Z)
```

## Multi-Limbed Creatures

For creatures without clear anatomical "front":
1. Define a logical front (sensor array, primary manipulators, etc.)
2. That front faces +Y
3. "Up" (dorsal surface, top) faces +Z
4. Limbs are positioned relative to body center

**Example: 8-legged spider**
```
        +Y (forward/head)
         |
    L3 L2|L1 L0
   ------+------  Body center at origin
    R3 R2|R1 R0
         |
        -Y (rear)

Legs numbered 0-3 from front to back.
L = left (-X side), R = right (+X side)
```

## Quick Reference Table

| Asset Type | Origin | Faces | Notes |
|------------|--------|-------|-------|
| Humanoid | Feet (Z=0) | +Y | T-pose, symmetric on X=0 |
| Quadruped | Feet (Z=0) | +Y (head) | Spine along Y |
| Spider | Body center | +Y (front) | Legs symmetric on X |
| Static prop | Center/base | +Y (front) | Context-dependent origin |
| Vehicle | Base | +Y (forward) | Driver faces +Y |

## See Also

- [Character Specs](../spec-reference/character.md) - Skeletal mesh parameters
- [Animation Specs](../spec-reference/animation.md) - Animation keyframes
- [Mesh Specs](../spec-reference/mesh.md) - Static mesh parameters
```

**Step 3: Verify file creation**

Run: `cat docs/conventions/coordinate-system.md | head -20`
Expected: First 20 lines of the document

**Step 4: Commit**

```bash
git add docs/conventions/coordinate-system.md
git commit -m "docs: add coordinate system conventions reference

Establishes canonical Z-up, Y-forward coordinate system based on
Blender convention. Documents position, rotation, skeleton, and
mesh conventions for humanoids, quadrupeds, and multi-limbed creatures.

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>"
```

---

## Task 2: Add Convention Header to Character Spec Reference

Update the character spec documentation to reference coordinate conventions.

**Files:**
- Modify: `docs/spec-reference/character.md:1-20`

**Step 1: Read current file header**

Run: `head -25 docs/spec-reference/character.md`

**Step 2: Add convention header after the property table**

Insert after line 11 (after the property table):

```markdown

> **Coordinate System:** SpecCade uses Z-up, Y-forward convention (Blender standard). See [Coordinate System Conventions](../conventions/coordinate-system.md) for details.
> - Character origin at feet (Z=0)
> - Character faces +Y in rest pose
> - Left side at -X, right side at +X
```

**Step 3: Update bone position field descriptions**

Find the "Custom Skeleton" section and update field descriptions to include axis meanings:

In the `head` field description, change:
```
| `head` | `[f64; 3]` | No | Head position [X, Y, Z] |
```
to:
```
| `head` | `[f64; 3]` | No | Head position [X (right), Y (forward), Z (up)] in meters |
```

Similarly for `tail`:
```
| `tail` | `[f64; 3]` | No | Tail position [X (right), Y (forward), Z (up)] in meters |
```

**Step 4: Update offset field in BodyPartMesh**

Change:
```
| `offset` | `[f64; 3]` | No | Position offset from bone |
```
to:
```
| `offset` | `[f64; 3]` | No | Position offset from bone [X (right), Y (forward), Z (up)] |
```

And `rotation`:
```
| `rotation` | `[f64; 3]` | No | Euler rotation [X (pitch), Y (roll), Z (yaw)] in degrees |
```

**Step 5: Verify changes**

Run: `grep -n "Coordinate System" docs/spec-reference/character.md`
Expected: Line showing the convention header

**Step 6: Commit**

```bash
git add docs/spec-reference/character.md
git commit -m "docs(character): add coordinate convention header and axis labels

References the new coordinate-system.md and adds axis meaning labels
to all [X, Y, Z] field descriptions.

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>"
```

---

## Task 3: Add Convention Header to Animation Spec Reference

Update the animation spec documentation to reference coordinate conventions.

**Files:**
- Modify: `docs/spec-reference/animation.md:1-20`

**Step 1: Read current file header**

Run: `head -15 docs/spec-reference/animation.md`

**Step 2: Add convention header after the property table**

Insert after line 10 (after the determinism row):

```markdown

> **Coordinate System:** SpecCade uses Z-up, Y-forward convention. See [Coordinate System Conventions](../conventions/coordinate-system.md).
> - Rotation X = pitch (nod), Y = roll (tilt), Z = yaw (turn)
> - Positive rotation = counter-clockwise (right-hand rule)
> - Root motion along +Y = forward movement
```

**Step 3: Update BoneTransform field descriptions**

Find the BoneTransform table (around line 104-107) and update:

Change:
```
| `position` | `[f64; 3]` | No | Position offset [X, Y, Z] |
| `rotation` | `[f64; 3]` | No | Euler rotation [X, Y, Z] in degrees |
| `scale` | `[f64; 3]` | No | Scale [X, Y, Z] |
```
to:
```
| `position` | `[f64; 3]` | No | Position offset [X (right), Y (forward), Z (up)] in meters |
| `rotation` | `[f64; 3]` | No | Euler rotation [X (pitch), Y (roll), Z (yaw)] in degrees |
| `scale` | `[f64; 3]` | No | Scale [X (width), Y (depth), Z (height)] |
```

**Step 4: Verify changes**

Run: `grep -n "Coordinate System" docs/spec-reference/animation.md`
Expected: Line showing the convention header

**Step 5: Commit**

```bash
git add docs/spec-reference/animation.md
git commit -m "docs(animation): add coordinate convention header and axis labels

References coordinate-system.md and adds axis meaning labels to
position, rotation, and scale field descriptions.

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>"
```

---

## Task 4: Add Convention Header to Mesh Spec Reference

Update the static mesh spec documentation to reference coordinate conventions.

**Files:**
- Modify: `docs/spec-reference/mesh.md:1-20`

**Step 1: Read current file header**

Run: `head -15 docs/spec-reference/mesh.md`

**Step 2: Add convention header after the property table**

Insert after line 10:

```markdown

> **Coordinate System:** SpecCade uses Z-up, Y-forward convention. See [Coordinate System Conventions](../conventions/coordinate-system.md).
> - Mesh "front" faces +Y
> - Mesh "top" faces +Z
> - Dimensions are [X (width), Y (depth), Z (height)]
```

**Step 3: Update dimensions field description**

Find the Main Parameters table and update:

Change:
```
| `dimensions` | `[f64; 3]` | Yes | Dimensions [X, Y, Z] in Blender units |
```
to:
```
| `dimensions` | `[f64; 3]` | Yes | Dimensions [X (width), Y (depth), Z (height)] in meters |
```

**Step 4: Update Array modifier offset description**

Change:
```
| `offset` | `[f64; 3]` | Yes | Offset between copies |
```
to:
```
| `offset` | `[f64; 3]` | Yes | Offset between copies [X (right), Y (forward), Z (up)] |
```

**Step 5: Verify changes**

Run: `grep -n "Coordinate System" docs/spec-reference/mesh.md`
Expected: Line showing the convention header

**Step 6: Commit**

```bash
git add docs/spec-reference/mesh.md
git commit -m "docs(mesh): add coordinate convention header and axis labels

References coordinate-system.md and adds axis meaning labels to
dimensions and offset field descriptions.

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>"
```

---

## Task 5: Add Coordinate System Metadata to Stdlib Dump

Extend the stdlib dump command to include coordinate system metadata.

**Files:**
- Modify: `crates/speccade-cli/src/commands/stdlib/mod.rs:52-77`

**Step 1: Read current StdlibDump struct**

Run: `grep -A 10 "pub struct StdlibDump" crates/speccade-cli/src/commands/stdlib/mod.rs`

**Step 2: Write failing test**

Add to the tests module at the bottom of the file:

```rust
#[test]
fn test_dump_includes_coordinate_system() {
    let dump = StdlibDump::new();
    let json = serde_json::to_string(&dump).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

    let coord = parsed.get("coordinate_system").expect("coordinate_system field missing");
    assert_eq!(coord.get("handedness").unwrap(), "right");
    assert_eq!(coord.get("up").unwrap(), "+Z");
    assert_eq!(coord.get("forward").unwrap(), "+Y");
    assert_eq!(coord.get("units").unwrap(), "meters");
}
```

**Step 3: Run test to verify it fails**

Run: `cargo test -p speccade-cli test_dump_includes_coordinate_system`
Expected: FAIL with "coordinate_system field missing"

**Step 4: Add CoordinateSystem struct**

Add after the RangeInfo struct (around line 119):

```rust
/// Coordinate system metadata for the stdlib.
#[derive(Debug, Serialize)]
pub struct CoordinateSystem {
    pub handedness: String,
    pub up: String,
    pub forward: String,
    pub right: String,
    pub units: String,
    pub rotation_order: String,
    pub rotation_units: String,
}

impl Default for CoordinateSystem {
    fn default() -> Self {
        Self {
            handedness: "right".into(),
            up: "+Z".into(),
            forward: "+Y".into(),
            right: "+X".into(),
            units: "meters".into(),
            rotation_order: "XYZ".into(),
            rotation_units: "degrees".into(),
        }
    }
}
```

**Step 5: Update StdlibDump struct**

Change:
```rust
pub struct StdlibDump {
    pub stdlib_version: String,
    pub functions: Vec<FunctionInfo>,
}
```
to:
```rust
pub struct StdlibDump {
    pub stdlib_version: String,
    pub coordinate_system: CoordinateSystem,
    pub functions: Vec<FunctionInfo>,
}
```

**Step 6: Update StdlibDump::new()**

Change the Self construction:
```rust
Self {
    stdlib_version: STDLIB_VERSION.to_string(),
    coordinate_system: CoordinateSystem::default(),
    functions,
}
```

**Step 7: Run test to verify it passes**

Run: `cargo test -p speccade-cli test_dump_includes_coordinate_system`
Expected: PASS

**Step 8: Run all stdlib tests**

Run: `cargo test -p speccade-cli commands::stdlib`
Expected: All tests pass

**Step 9: Commit**

```bash
git add crates/speccade-cli/src/commands/stdlib/mod.rs
git commit -m "feat(stdlib): add coordinate_system metadata to dump

Includes handedness, axes, units, rotation order in stdlib dump JSON.
LLMs can read this to understand SpecCade's coordinate conventions.

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>"
```

---

## Task 6: Update Stdlib Reference Documentation

Add coordinate system section to the stdlib reference docs.

**Files:**
- Modify: `docs/stdlib-reference.md`

**Step 1: Read current file structure**

Run: `head -50 docs/stdlib-reference.md`

**Step 2: Add coordinate system section after the overview**

Insert a new section early in the document (after any intro/overview):

```markdown
## Coordinate System

All stdlib functions use SpecCade's canonical coordinate system:

| Property | Value |
|----------|-------|
| Handedness | Right-handed |
| Up | +Z |
| Forward | +Y |
| Right | +X |
| Units | Meters |
| Rotation Order | XYZ (Euler) |
| Rotation Units | Degrees |

**Key conventions:**
- `[X, Y, Z]` positions: `[right, forward, up]`
- `[X, Y, Z]` rotations: `[pitch, roll, yaw]`
- Character origin at feet, facing +Y
- Positive rotation = counter-clockwise (right-hand rule)

See [Coordinate System Conventions](conventions/coordinate-system.md) for full details.
```

**Step 3: Verify changes**

Run: `grep -n "Coordinate System" docs/stdlib-reference.md`
Expected: Line showing the new section

**Step 4: Commit**

```bash
git add docs/stdlib-reference.md
git commit -m "docs(stdlib): add coordinate system quick reference

Adds table summarizing coordinate conventions at top of stdlib reference.

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>"
```

---

## Task 7: Add Convention Link to docs/README.md

Ensure the conventions doc is discoverable from the docs index.

**Files:**
- Modify: `docs/README.md`

**Step 1: Read current docs README**

Run: `cat docs/README.md`

**Step 2: Add conventions section**

Add a new section or link in the appropriate location:

```markdown
## Conventions

- [Coordinate System](conventions/coordinate-system.md) - Axis conventions for meshes and animations
```

**Step 3: Commit**

```bash
git add docs/README.md
git commit -m "docs: add coordinate system to docs index

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>"
```

---

## Task 8: Audit Golden Specs for Convention Compliance

Check if existing golden specs follow the coordinate conventions.

**Files:**
- Read: `golden/speccade/specs/skeletal_mesh/*.json`
- Read: `golden/speccade/specs/skeletal_animation/*.json`

**Step 1: Check humanoid specs for correct orientation**

Run: `cat golden/speccade/specs/skeletal_mesh/preset_humanoid.json | jq '.recipe.params.body_parts[0]'`

Verify that body parts use Z for vertical positioning (spine, head at positive Z).

**Step 2: Check animation specs for correct rotation semantics**

Run: `cat golden/speccade/specs/skeletal_animation/idle_breathe.json | jq '.recipe.params.keyframes[1].bones'`

Verify rotations use X for pitch (forward/back nod).

**Step 3: Document any violations**

If violations found, create a list of specs needing updates.

**Step 4: (If needed) Update golden specs**

Only if clear violations exist. Otherwise, note "All golden specs compliant" in commit.

**Step 5: Commit (if changes made)**

```bash
git add golden/
git commit -m "fix(golden): update specs to follow coordinate conventions

[List any specific changes made]

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>"
```

Or if no changes needed:

```bash
# No commit needed - add note to task completion
echo "Golden spec audit complete - all specs follow conventions"
```

---

## Task 9: Final Verification

Verify all documentation links work and stdlib dump includes new metadata.

**Step 1: Build and test**

Run: `cargo build -p speccade-cli && cargo test -p speccade-cli`
Expected: Build and tests pass

**Step 2: Verify stdlib dump**

Run: `cargo run -p speccade-cli -- stdlib dump --format json | jq '.coordinate_system'`
Expected: JSON object with handedness, up, forward, right, units, rotation_order, rotation_units

**Step 3: Verify documentation links**

Check that all cross-references resolve:
- `docs/conventions/coordinate-system.md` exists
- Links in spec-reference docs point to correct files
- `docs/README.md` links to conventions

**Step 4: Final commit (if any loose changes)**

```bash
git status
# If clean, proceed
# If changes exist, commit them
```

---

## Success Criteria

1. `docs/conventions/coordinate-system.md` exists with complete convention documentation
2. `docs/spec-reference/character.md` has coordinate convention header and axis labels
3. `docs/spec-reference/animation.md` has coordinate convention header and axis labels
4. `docs/spec-reference/mesh.md` has coordinate convention header and axis labels
5. `speccade stdlib dump` output includes `coordinate_system` object
6. `docs/stdlib-reference.md` has coordinate system quick reference section
7. `docs/README.md` links to coordinate conventions
8. All golden specs follow documented conventions (or violations documented)
9. All tests pass
