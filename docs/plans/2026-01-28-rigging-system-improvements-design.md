# Rigging System Improvements Design

**Date:** 2026-01-28
**Status:** Draft
**Author:** Claude + User

## Overview

Improve the animation rigging system by adding missing professional features (IK/FK snapping, space switching, finger controls), expanding skeleton presets, and creating a verification pipeline with GIF preview output.

## Problem Statement

The current rigging system has comprehensive Rust types and Python implementations for features like foot systems, aim constraints, twist bones, and bone constraints. However:

1. **No verification** - Zero specs exercise these features, so we can't confirm they work
2. **Missing professional features** - IK/FK snapping, space switching, finger controls
3. **Limited skeleton presets** - Only `humanoid_basic_v1` exists (20 bones, no fingers)
4. **No visual preview** - Can't render animations as GIFs for quick verification

## Goals

1. Add IK/FK snapping for seamless animation workflow
2. Add space switching for dynamic parent changes
3. Add finger curl/spread controls with anatomical correctness
4. Create two new skeleton presets (detailed and game-optimized)
5. Build GIF preview pipeline for visual verification
6. Create verification specs that exercise all rigging features

---

## New Feature: IK/FK Snapping

### Purpose

Allow instant pose transfer between IK and FK control without popping. Essential for switching control modes mid-animation.

### Rust Types

```rust
// New file: crates/speccade-spec/src/recipe/animation/ikfk_switch.rs

/// IK/FK switch configuration for a limb
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IkFkSwitch {
    /// Name of this switch (e.g., "arm_l_ikfk")
    pub name: String,
    /// The IK chain this switch controls
    pub ik_chain: String,
    /// FK bones in order (root to tip)
    pub fk_bones: Vec<String>,
    /// Default mode at frame 0
    #[serde(default)]
    pub default_mode: IkFkMode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IkFkMode {
    #[default]
    Ik,
    Fk,
}

/// Keyframe for IK/FK switch with optional snapping
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IkFkKeyframe {
    pub frame: u32,
    pub mode: IkFkMode,
    /// If true, snap pose before switching (no pop)
    #[serde(default)]
    pub snap: bool,
}
```

### Python Implementation

1. Create custom property on armature for each switch (0.0 = FK, 1.0 = IK)
2. Add drivers that set IK constraint influence based on property
3. Implement snap functions:
   - `snap_ik_to_fk()`: Copy FK bone rotations to match current IK pose
   - `snap_fk_to_ik()`: Move IK target/pole to match current FK pose

### Integration

Add to `RigSetup`:
```rust
pub struct RigSetup {
    // ... existing fields ...
    /// IK/FK switches for limbs
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub ikfk_switches: Vec<IkFkSwitch>,
}
```

---

## New Feature: Space Switching

### Purpose

Dynamically change what a bone's transforms are relative to. Essential for:
- Picking up objects (hand switches to object space)
- Planting hands on walls (hand switches to world space)
- Two-character interactions

### Rust Types

```rust
// New file: crates/speccade-spec/src/recipe/animation/space_switch.rs

/// Defines available parent spaces for a bone
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpaceSwitch {
    /// Name of this switch (e.g., "hand_l_space")
    pub name: String,
    /// The bone being controlled
    pub bone: String,
    /// Available parent spaces
    pub spaces: Vec<ParentSpace>,
    /// Default space index (0-based)
    #[serde(default)]
    pub default_space: usize,
}

/// A parent space option
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ParentSpace {
    /// Display name (e.g., "World", "Root", "Head")
    pub name: String,
    /// Space type
    #[serde(flatten)]
    pub kind: SpaceKind,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SpaceKind {
    /// World space (no parent)
    World,
    /// Root bone space
    Root,
    /// Relative to another bone
    Bone { bone: String },
    /// Relative to an external object/empty
    Object { object: String },
}

/// Keyframe for space switch
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpaceSwitchKeyframe {
    pub frame: u32,
    /// Index into spaces array
    pub space: usize,
    /// If true, maintain visual position when switching
    #[serde(default = "default_true")]
    pub maintain_offset: bool,
}
```

### Python Implementation

1. Create multiple CHILD_OF constraints on the bone (one per space)
2. Use custom property + drivers to blend influence (only one active at a time)
3. On space switch with `maintain_offset: true`, calculate and apply offset matrix

### Common Presets

| Preset | Bone | Spaces |
|--------|------|--------|
| `hand_space` | hand_l/r | World, Root, Head, Hips |
| `foot_space` | ik_foot_l/r | World, Root, Hips |
| `head_space` | head | World, Chest, Root |

---

## New Feature: Finger Controls

### Purpose

Control finger poses with simple curl/spread values instead of per-bone rotations.

### Anatomical Convention

Following standard anatomical convention:
- **Flexion** (curling toward palm) = **positive** rotation
- **Extension** (straightening) = **negative** or zero

| Curl Value | Anatomical Term | Visual |
|------------|-----------------|--------|
| 0.0 | Full extension | Straight finger |
| 0.5 | Mid-flexion | Relaxed curl |
| 1.0 | Full flexion | Tight fist |
| -0.2 | Hyperextension | Bent backward (limited) |

### Bone Roll Requirement

For curl to work correctly, skeleton presets must define bone roll explicitly. Finger bones should have roll set so positive X rotation = flexion.

### Rust Types

```rust
// New file: crates/speccade-spec/src/recipe/animation/finger_controls.rs

/// Finger control configuration for a hand
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FingerControls {
    /// Which hand ("hand_l" or "hand_r")
    pub hand_bone: String,
    /// Control mode
    #[serde(default)]
    pub mode: FingerControlMode,
    /// Finger definitions (auto-populated from skeleton preset if omitted)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub fingers: Vec<FingerDefinition>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FingerControlMode {
    /// Direct FK control per bone
    #[default]
    Fk,
    /// Curl/spread drivers
    Curl,
    /// IK targets for fingertips
    Ik,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FingerDefinition {
    /// Finger name (thumb, index, middle, ring, pinky)
    pub name: String,
    /// Bone chain from base to tip
    pub bones: Vec<String>,
    /// Curl axis per joint (usually X)
    #[serde(default = "default_curl_axis")]
    pub curl_axis: ConstraintAxis,
    /// Max curl angle per joint in degrees
    #[serde(default = "default_curl_angles")]
    pub curl_angles: Vec<f64>,
}

/// Keyframe for finger pose
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FingerKeyframe {
    pub frame: u32,
    /// Per-finger curl values (0.0 = straight, 1.0 = full curl)
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub curl: HashMap<String, f64>,
    /// Overall spread value (-1.0 = together, 0.0 = neutral, 1.0 = spread)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub spread: Option<f64>,
}
```

### Python Implementation

1. Create custom properties on hand bone: `curl_thumb`, `curl_index`, etc. (0-1 range)
2. Create `spread` property (-1 to 1)
3. Add drivers on each finger joint: `rotation_euler.x = curl_value * max_angle`
4. Spread rotates fingers on Y axis based on finger index

### Preset Finger Poses

| Pose Name | Curl Values | Use Case |
|-----------|-------------|----------|
| `open` | all 0.0 | Relaxed hand |
| `fist` | all 1.0 | Closed fist |
| `point` | index 0.0, others 1.0 | Pointing |
| `grip` | all 0.7 | Holding object |
| `relax` | all 0.2 | Natural rest |

---

## New Skeleton Presets

### Add Bone Roll to Skeleton Definition

```rust
// In skeleton.rs - update SkeletonBone
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SkeletonBone {
    pub bone: String,
    pub head: Option<[f64; 3]>,
    pub tail: Option<[f64; 3]>,
    pub parent: Option<String>,
    pub mirror: Option<String>,
    /// Bone roll in degrees (rotation around bone axis)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub roll: Option<f64>,
}
```

### humanoid_detailed_v1 (~52 bones)

Full humanoid with 3-bone fingers, thumbs, and toes.

```
root
└── hips
    ├── spine
    │   └── chest
    │       ├── neck
    │       │   └── head
    │       ├── shoulder_l
    │       │   └── upper_arm_l
    │       │       └── lower_arm_l
    │       │           └── hand_l
    │       │               ├── thumb_01_l → thumb_02_l → thumb_03_l
    │       │               ├── index_01_l → index_02_l → index_03_l
    │       │               ├── middle_01_l → middle_02_l → middle_03_l
    │       │               ├── ring_01_l → ring_02_l → ring_03_l
    │       │               └── pinky_01_l → pinky_02_l → pinky_03_l
    │       └── shoulder_r (mirror of left)
    ├── upper_leg_l
    │   └── lower_leg_l
    │       └── foot_l
    │           └── toe_l
    └── upper_leg_r (mirror of left)
```

**Bone count breakdown:**
- Core: 6 (root, hips, spine, chest, neck, head)
- Arms: 8 (shoulder, upper_arm, lower_arm, hand × 2)
- Fingers: 30 (5 fingers × 3 bones × 2 hands)
- Legs: 6 (upper_leg, lower_leg, foot × 2)
- Toes: 2 (toe × 2)
- **Total: 52 bones**

### humanoid_game_v1 (~32 bones)

Game-optimized with twist bones and simplified fingers.

```
root
└── hips
    ├── spine
    │   └── chest
    │       ├── neck
    │       │   └── head
    │       ├── shoulder_l
    │       │   └── upper_arm_l
    │       │       ├── upper_arm_twist_l
    │       │       └── lower_arm_l
    │       │           ├── lower_arm_twist_l
    │       │           └── hand_l
    │       │               ├── thumb_l (1 bone)
    │       │               ├── index_l (1 bone)
    │       │               ├── middle_l (1 bone)
    │       │               ├── ring_l (1 bone)
    │       │               └── pinky_l (1 bone)
    │       └── shoulder_r (mirror)
    ├── upper_leg_l
    │   ├── upper_leg_twist_l
    │   └── lower_leg_l
    │       ├── lower_leg_twist_l
    │       └── foot_l
    │           └── toe_l
    └── upper_leg_r (mirror)
```

**Bone count breakdown:**
- Core: 6 (root, hips, spine, chest, neck, head)
- Arms: 12 (shoulder, upper_arm, upper_arm_twist, lower_arm, lower_arm_twist, hand × 2)
- Fingers: 10 (5 fingers × 1 bone × 2 hands)
- Legs: 10 (upper_leg, upper_leg_twist, lower_leg, lower_leg_twist, foot × 2)
- Toes: 2 (toe × 2)
- **Total: 40 bones** (correction from earlier estimate)

---

## GIF Preview Pipeline

### Purpose

Render skeletal animations directly to GIF in Blender for quick visual verification. LLMs can view GIFs to validate rig setup.

### Rust Types

```rust
// Add to rig.rs or new file: preview.rs

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PreviewRender {
    /// Camera angle preset
    #[serde(default)]
    pub camera: CameraPreset,
    /// Output frame size [width, height]
    #[serde(default = "default_preview_size")]
    pub size: [u32; 2],
    /// Render every Nth frame (1 = all, 2 = half, etc.)
    #[serde(default = "default_frame_step")]
    pub frame_step: u32,
    /// Background color [R, G, B, A] 0-1 range
    #[serde(default = "default_background")]
    pub background: [f32; 4],
    /// Reference to mesh spec to include (path or asset_id)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mesh: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CameraPreset {
    #[default]
    ThreeQuarter,
    Front,
    Side,
    Back,
    Top,
}

fn default_preview_size() -> [u32; 2] { [256, 256] }
fn default_frame_step() -> u32 { 2 }
fn default_background() -> [f32; 4] { [0.2, 0.2, 0.2, 1.0] }
```

### Camera Presets

| Preset | Position | Best For |
|--------|----------|----------|
| `three_quarter` | 45° front-right, above | General character review |
| `front` | Directly in front | Face/silhouette |
| `side` | 90° side view | Walk cycle timing |
| `back` | Behind character | Back detail |
| `top` | Above looking down | Footwork patterns |

### Python Implementation

```python
def render_animation_preview_gif(
    armature: bpy.types.Object,
    mesh_obj: Optional[bpy.types.Object],
    output_path: Path,
    camera_preset: str,
    size: Tuple[int, int],
    fps: int,
    frame_start: int,
    frame_end: int,
    frame_step: int = 1,
    background_color: Tuple[float, float, float, float] = (0.2, 0.2, 0.2, 1.0),
) -> Dict[str, Any]:
    """Render animation to GIF using Blender's FFMPEG output."""

    # 1. Setup camera at preset position
    camera = setup_preview_camera(armature, camera_preset)

    # 2. Configure render settings
    scene = bpy.context.scene
    scene.render.resolution_x = size[0]
    scene.render.resolution_y = size[1]
    scene.render.film_transparent = False
    scene.frame_start = frame_start
    scene.frame_end = frame_end
    scene.frame_step = frame_step
    scene.render.fps = fps

    # 3. Set background color
    world = bpy.data.worlds.new("PreviewWorld")
    world.use_nodes = False
    world.color = background_color[:3]
    scene.world = world

    # 4. Set output format to FFMPEG GIF
    scene.render.image_settings.file_format = 'FFMPEG'
    scene.render.ffmpeg.format = 'GIF'

    # 5. Render animation
    scene.render.filepath = str(output_path)
    bpy.ops.render.render(animation=True)

    return {
        "preview_gif": str(output_path),
        "frames_rendered": (frame_end - frame_start) // frame_step + 1,
    }
```

### Output Structure

```json
{
  "outputs": [
    { "path": "walk_cycle.glb", "kind": "animation" },
    { "path": "walk_cycle.blend", "kind": "blend" },
    { "path": "walk_cycle_preview.gif", "kind": "preview" }
  ]
}
```

---

## Verification Specs

Create specs in `golden/speccade/specs/rigging_verification/` that exercise each feature:

| Spec File | Features Tested | Expected Visual |
|-----------|-----------------|-----------------|
| `foot_roll_test.star` | FootSystem | Foot rolls heel→ball→toe during step |
| `aim_constraint_test.star` | AimConstraint | Head tracks moving target |
| `twist_bones_test.star` | TwistBone | Smooth arm twist distribution |
| `bone_constraints_test.star` | Hinge/Ball/Planar/Soft | Joints respect limits |
| `ikfk_snap_test.star` | IkFkSwitch | Seamless IK↔FK transition |
| `space_switch_test.star` | SpaceSwitch | Hand maintains position on parent change |
| `finger_curl_test.star` | FingerControls | Hand opens and closes smoothly |
| `full_rig_test.star` | All combined | Complete animation demo |

### Each Verification Spec Outputs

- `{name}.glb` - Exported animation
- `{name}.blend` - Inspectable Blender file (save_blend: true)
- `{name}_preview.gif` - Visual verification
- `metrics.json` - Validation metrics

---

## Implementation Order

### Phase 1: Foundation
1. Add `roll` field to `SkeletonBone`
2. Create `humanoid_detailed_v1` skeleton preset with bone rolls
3. Create `humanoid_game_v1` skeleton preset with twist bones
4. Implement GIF preview rendering in Python

### Phase 2: Core Features
5. Implement IK/FK snapping (Rust types + Python)
6. Implement space switching (Rust types + Python)
7. Implement finger controls (Rust types + Python)

### Phase 3: Verification
8. Create verification specs for existing features (foot_roll, aim, twist, constraints)
9. Create verification specs for new features (ikfk, space, fingers)
10. Create full_rig_test.star combining all features
11. Run all specs and verify GIF outputs

---

## Success Criteria

1. All verification specs generate without errors
2. GIF previews show expected visual behavior
3. .blend files can be opened and inspected
4. Bone constraints respect limits visually
5. IK/FK snapping produces no pops
6. Space switching maintains visual position
7. Finger curl values map correctly to anatomical flexion

---

## Open Questions

1. Should we support quadruped skeleton presets in this iteration?
2. Do we need face rig support?
3. Should finger IK (precise fingertip placement) be included?

---

## References

- [Rigify Documentation](https://docs.blender.org/manual/en/latest/addons/rigging/rigify.html)
- [Unity Humanoid Requirements](https://docs.unity3d.com/Manual/UsingHumanoidChars.html)
- Existing code: `crates/speccade-spec/src/recipe/animation/`
