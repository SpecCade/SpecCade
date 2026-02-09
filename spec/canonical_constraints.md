# SpecCade Canonical Constraints Checklist

Every visually verifiable rule extracted from SpecCade reference documentation.
Use this checklist to validate preview renders against canonical conventions.

> **Sources:** `docs/spec-reference/character.md`, `docs/spec-reference/mesh.md`,
> `docs/spec-reference/animation.md`, `docs/conventions/coordinate-system.md`,
> `docs/spec-reference/structural-metrics.md`, `docs/spec-reference/README.md`,
> `docs/rfcs/RFC-0012-sprite-assets.md`

---

## COORDINATE SYSTEM constraints

### Axis Orientation (Z-up, Y-forward, right-handed)

- [ ] +Z axis points UP (vertical / sky direction)
- [ ] +Y axis points FORWARD (direction a character or object faces)
- [ ] +X axis points RIGHT (character's right-hand side)
- [ ] System is right-handed (cross product: X x Y = Z)
- [ ] Positive rotation is counter-clockwise when looking down an axis toward the origin (right-hand rule)

### Rotation Semantics

- [ ] Rotation X component = pitch (nod forward / back)
- [ ] Rotation Y component = roll (tilt left / right)
- [ ] Rotation Z component = yaw (turn left / right)
- [ ] Euler order is XYZ (Blender default)
- [ ] All rotation values are specified in degrees, never radians

### Units

- [ ] All positions are in meters (Blender units)
- [ ] Dimensions arrays are ordered `[X width, Y depth, Z height]`
- [ ] Bone-relative fields are fractions of bone length (head-to-tail distance), not meters

### Origin Placement

- [ ] Humanoid character origin at feet (Z = 0, ground contact point)
- [ ] Quadruped character origin at feet (Z = 0)
- [ ] Multi-limbed creature (spider, etc.) origin at body center
- [ ] Static mesh origin at geometric center or base (context-dependent)
- [ ] Vehicle origin at base; driver faces +Y

### GLB Export Conversion

- [ ] Specs are authored in Blender convention (Z-up); GLB exporter converts to glTF convention (Y-up) automatically
- [ ] Spec authors never need to think in Y-up; all values in specs use Z-up, Y-forward

---

## CHARACTER constraints

### Humanoid Rest Pose (T-pose)

- [ ] Character stands upright; spine chain aligned with +Z axis
- [ ] Character faces +Y direction in rest pose
- [ ] Feet planted at Z = 0 (ground plane)
- [ ] Arms extended horizontally along X axis (left arm toward -X, right arm toward +X)
- [ ] Arms start OUTSIDE chest width (shoulder X > chest radius, e.g. X = 0.15 if chest radius = 0.14)
- [ ] Head at top of spine chain (Z approximately 1.5 - 1.8 for adult humanoid)
- [ ] Hips at approximately Z = 0.9 for adult humanoid

### Spine Chain

- [ ] Vertical along +Z: root -> hips -> spine -> chest -> neck -> head
- [ ] Each child bone head equals parent bone tail (no gaps in connected chains)
- [ ] Spine bones point from head to tail in +Z direction

### Arm Chain

- [ ] Arms extend horizontally along +/- X at constant Z
- [ ] Bone chain: shoulder -> upper_arm -> lower_arm -> hand
- [ ] Arm bones point from head to tail toward the hand (+-X direction)
- [ ] Shoulder bones defined with mesh (prevents floating arms with presets)

### Leg Chain

- [ ] Legs extend downward from hips (-Z direction)
- [ ] Bone chain: upper_leg -> lower_leg -> foot
- [ ] Leg bones point from head to tail toward the foot (-Z direction)

### Bone Naming Convention

- [ ] Left / right suffixes: `_l` and `_r`
- [ ] Hierarchy naming: `upper_arm_l` -> `lower_arm_l` -> `hand_l`
- [ ] Standard humanoid_basic_v1 bones: spine, chest, neck, head, shoulder_l/r, upper_arm_l/r, lower_arm_l/r, hand_l/r, upper_leg_l/r, lower_leg_l/r, foot_l/r

### Bilateral Symmetry

- [ ] Character is symmetric across YZ plane (X = 0 mirror plane)
- [ ] Left body parts at negative X, right body parts at positive X
- [ ] Left / right bone pairs have matching bone lengths (length_ratio approximately 1.0)
- [ ] Left / right bone pairs have matching mesh radii (radius_ratio approximately 1.0)
- [ ] structural.symmetry.x_axis > 0.9 for symmetric humanoids
- [ ] Mirror references (`{"mirror": "other_key"}`) produce visually mirrored geometry via mirrored skeleton bones

### Skeleton Coverage

- [ ] Bone coverage_ratio between 0.5 and 1.5 (mesh covers bone without excessive overhang)
- [ ] coverage_ratio < 0.5 indicates missing geometry on that bone
- [ ] Terminal bones present as expected: hands, feet, head for humanoid
- [ ] Hierarchy depth 4-6 for humanoid rigs; 2-3 for simple rigs

### Skinning (armature_driven_v1)

- [ ] Rigid skinning: each generated segment is 100% weighted to its bone
- [ ] Bridge edge loop vertices receive interpolated weights between parent and child bones
- [ ] Boolean helper shapes (`bool_shapes`) are NOT exported in final GLB

### Quadruped Orientation

- [ ] Spine runs along Y axis (head toward +Y, tail/rear toward -Y)
- [ ] Legs extend downward (-Z)
- [ ] Left legs at -X, right legs at +X
- [ ] Feet at Z = 0

### Multi-Limbed Creatures

- [ ] Logical front faces +Y
- [ ] Dorsal surface (top) faces +Z
- [ ] Limbs positioned relative to body center
- [ ] Spider legs numbered 0-3 from front to back; L = -X side, R = +X side

### Character Scale

- [ ] Human-scale characters: longest_dimension_m between 1.5 and 2.0
- [ ] extent Z dimension matches intended character height
- [ ] dominant_axis = "Z" for standing characters

---

## MESH constraints

### Topology

- [ ] Mesh is manifold (report field: `manifold: true`)
- [ ] No non-manifold edges or loose vertices
- [ ] Face normals point outward from enclosed volume
- [ ] No unintended overlapping or interpenetrating geometry

### Dimensions and Scale

- [ ] Dimensions in meters (Blender units = meters)
- [ ] Bounding box extent `[X, Y, Z]` matches intended size
- [ ] Small prop (handheld): longest_dimension < 0.1 m
- [ ] Medium prop (weapon, small furniture): longest_dimension 0.1 - 1.0 m
- [ ] Character-scale: longest_dimension 1.0 - 2.0 m
- [ ] Large-scale (vehicle, building): longest_dimension > 2.0 m
- [ ] `fits_in_1m_cube` and `fits_in_10cm_cube` flags match intended category

### Static Mesh Orientation

- [ ] "Front" of object faces +Y
- [ ] "Top" of object faces +Z
- [ ] Base primitives: cube, sphere, cylinder, cone, torus, plane, ico_sphere

### Normals

- [ ] Smooth shading for organic surfaces
- [ ] Flat shading for hard-surface / mechanical objects
- [ ] Auto-smooth by angle for mixed surfaces

### Modifiers (applied in order)

- [ ] Bevel: width, segments, and angle_limit produce expected edge rounding
- [ ] Subdivision: levels produce expected surface smoothness
- [ ] Decimate: ratio reduces triangle count to target
- [ ] Mirror: geometry mirrored across specified axes
- [ ] Array: copies placed at correct offset intervals
- [ ] Solidify: thickness and offset produce expected shell

### UV Mapping

- [ ] UV projection method matches geometry (box, cylinder, sphere, smart, lightmap)
- [ ] UV islands non-overlapping (unless intentionally mirrored)
- [ ] UV coverage efficient (minimal wasted atlas space)

### Bridge Edge Loops (Skeletal Meshes)

- [ ] Bridge only works between coaxial bones (same orientation)
- [ ] Profile segment counts must match exactly or be 2x multiples
- [ ] Both parent `connect_end` and child `connect_start` set to `"bridge"`
- [ ] Non-coaxial connections (chest -> shoulder, hips -> upper_leg) use caps, not bridge
- [ ] Both bones must have mesh definitions (not mirror references) for bridging

### Bone Mesh Profiles

- [ ] Profile segment count specified (e.g., `circle(8)`, `hexagon(8)`)
- [ ] profile_radius in bone-relative units (fraction of bone length)
- [ ] Taper < 1.0 narrows toward tail; taper > 1.0 widens toward tail

### Geometry Metrics

- [ ] aspect_ratios match intended proportions
- [ ] dominant_axis matches object orientation ("Z" for standing, "Y" for long)
- [ ] convex_hull_ratio < 0.5 for complex silhouettes (characters with limbs)
- [ ] convex_hull_ratio > 0.8 for simple shapes (cubes, spheres)

---

## ANIMATION constraints

### Root Motion

- [ ] Root motion along +Y = forward movement
- [ ] Root motion along +Z = vertical (jump / fall) movement

### Bone Transform Semantics

- [ ] Position offset: [X (right), Y (forward), Z (up)] in meters
- [ ] Rotation: [X (pitch), Y (roll), Z (yaw)] in degrees
- [ ] At least one transform property per bone per keyframe

### Hinge Joints

- [ ] Hinge joints (knees, elbows) bend only along their intended axis
- [ ] hinge_axis_violations = 0 for correct joint movement

### Looping Animations (Cycle Rules)

- [ ] First and last keyframes have IDENTICAL bone transforms for seamless loop
- [ ] `loop: true` set in spec for cyclic animations

### Non-Looping Animations

- [ ] `loop: false` set in spec
- [ ] Duration appropriate for action type

---

## SPRITE constraints

### Spritesheet Atlas

- [ ] Padding >= 2 pixels between frames (mip-safe gutter)
- [ ] All frames fit within atlas dimensions
- [ ] Frame IDs unique within spritesheet

### Sprite Render from Mesh

- [ ] All rotation angles represented in atlas frames
- [ ] Camera type: orthographic (no perspective distortion)
- [ ] Full model silhouette visible in each frame

### Alpha Channel

- [ ] Background fully transparent (alpha = 0) for sprite frames
- [ ] No semi-transparent fringe artifacts around sprite edges

---

## GENERAL constraints (preview renders)

### Camera

- [ ] Asset centered in frame
- [ ] Full body/geometry visible (no clipping at frame edges)
- [ ] Appropriate headroom above subject

### Lighting (3D Previews)

- [ ] Sufficient illumination to see all surface detail
- [ ] No pure black shadows obscuring geometry

### Background

- [ ] Neutral background for asset previews
- [ ] Background does not interfere with silhouette reading

### Silhouette Clarity

- [ ] Outline clearly readable against background
- [ ] Limbs distinguishable from torso in rest pose (characters)
- [ ] No self-occlusion hiding critical features

---

## DETERMINISM constraints

### Tier 2 (Metric validation - Blender assets)

- [ ] Triangle count within tolerance across regeneration
- [ ] Bone count identical across regeneration
- [ ] Bounding box dimensions within tolerance
- [ ] Same seed + same spec = metrics within tolerance

---

## Usage

1. Generate the asset: `speccade generate --spec file.star`
2. Generate preview grid: `speccade preview-grid --spec file.star`
3. Walk through the relevant constraint categories above
4. If any checkbox fails, revise the spec and/or fix the generator, then regenerate
