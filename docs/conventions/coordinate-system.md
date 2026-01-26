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
