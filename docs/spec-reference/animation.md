# Animation (Skeletal Animation) Specs

Skeletal animations are keyframe-based animation clips for skeletal meshes.

| Property | Value |
|----------|-------|
| Asset Type | `skeletal_animation` |
| Recipe Kinds | `skeletal_animation.blender_clip_v1`, `skeletal_animation.blender_rigged_v1` |
| Output Formats | `glb` |
| Determinism | Tier 2 (metric validation) |

## Recipe Kind Selection: Clip vs Rigged

SpecCade provides two recipe kinds for skeletal animation, each with a distinct purpose:

| Recipe Kind | Use Case | Features |
|-------------|----------|----------|
| `blender_clip_v1` | Simple FK animations | Direct bone keyframes, time-based |
| `blender_rigged_v1` | IK-enabled animations | IK targets, poses, phases, procedural layers |

**Choose `blender_clip_v1` when:**
- Animating with direct bone rotations (forward kinematics)
- Creating simple looping animations (idle, breathing)
- Animating upper body only (attacks, gestures)
- You want the simplest possible spec

**Choose `blender_rigged_v1` when:**
- Using IK targets for feet/hands (walk cycles, reaching)
- Defining named poses and animation phases
- Using procedural layers (breathing, sway)
- Need IK/FK blending or foot roll systems

**Schema Enforcement:** Validation will reject IK-specific fields (`rig_setup`, `poses`, `phases`, `ik_keyframes`, `procedural_layers`, etc.) in `blender_clip_v1` specs with a clear error message directing you to use `blender_rigged_v1` instead.

## SSOT (Source Of Truth)

- Rust types: `crates/speccade-spec/src/recipe/animation/`
- Golden specs: `golden/speccade/specs/skeletal_animation/`
- CLI validation: `speccade validate --spec file.json`

---

## blender_clip_v1: Simple Keyframe Animation

The `skeletal_animation.blender_clip_v1` recipe creates animation clips with direct bone keyframes.

### Main Parameters

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `skeleton_preset` | string | Yes | Skeleton rig to animate |
| `clip_name` | string | Yes | Name of the animation clip |
| `duration_seconds` | f64 | Yes | Animation duration |
| `fps` | u8 | Yes | Frames per second |
| `loop` | bool | No | Whether animation loops (default: false) |
| `keyframes` | array | Yes | Keyframe definitions |
| `interpolation` | string | No | Interpolation mode (default: `linear`) |
| `export` | object | No | Export settings |

### Skeleton Presets

Animations must target a skeleton preset. Currently available:

| Value | Description |
|-------|-------------|
| `humanoid_basic_v1` | Basic humanoid skeleton (20 bones) |

See [Character Specs](character.md) for bone names.

### Keyframes

Keyframes define bone transforms at specific times:

```json
"keyframes": [
  {
    "time": 0.0,
    "bones": {
      "spine": { "rotation": [0, 0, 0] },
      "chest": { "rotation": [0, 0, 0] }
    }
  },
  {
    "time": 1.0,
    "bones": {
      "spine": { "rotation": [2, 0, 0] },
      "chest": { "rotation": [4, 0, 0] }
    }
  }
]
```

#### AnimationKeyframe

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `time` | f64 | Yes | Time in seconds |
| `bones` | object | Yes | Map of bone name to transform |

#### BoneTransform

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `position` | `[f64; 3]` | No | Position offset [X, Y, Z] |
| `rotation` | `[f64; 3]` | No | Euler rotation [X, Y, Z] in degrees |
| `scale` | `[f64; 3]` | No | Scale [X, Y, Z] |

At least one transform property should be set per bone.

### Interpolation Modes

| Value | Description |
|-------|-------------|
| `linear` | Linear interpolation (default) |
| `bezier` | Bezier curve interpolation |
| `constant` | No interpolation (step/hold) |

### Export Settings

```json
"export": {
  "bake_transforms": true,
  "optimize_keyframes": false,
  "separate_file": false,
  "save_blend": false
}
```

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `bake_transforms` | bool | true | Bake all transforms to keyframes |
| `optimize_keyframes` | bool | false | Remove redundant keyframes |
| `separate_file` | bool | false | Export as separate file |
| `save_blend` | bool | false | Save .blend file alongside GLB |

## Example Spec

```json
{
  "spec_version": 1,
  "asset_id": "idle_breathe",
  "asset_type": "skeletal_animation",
  "license": "CC0-1.0",
  "seed": 8001,
  "description": "Idle breathing animation",
  "outputs": [
    {
      "kind": "primary",
      "format": "glb",
      "path": "idle_breathe.glb"
    }
  ],
  "recipe": {
    "kind": "skeletal_animation.blender_clip_v1",
    "params": {
      "skeleton_preset": "humanoid_basic_v1",
      "clip_name": "idle_breathe",
      "duration_seconds": 2.0,
      "fps": 30,
      "loop": true,
      "interpolation": "bezier",
      "keyframes": [
        {
          "time": 0.0,
          "bones": {
            "spine": { "rotation": [0, 0, 0] },
            "chest": { "rotation": [0, 0, 0] },
            "head": { "rotation": [0, 0, 0] }
          }
        },
        {
          "time": 1.0,
          "bones": {
            "spine": { "rotation": [1, 0, 0] },
            "chest": { "rotation": [2, 0, 0] },
            "head": { "rotation": [-1, 0, 0] }
          }
        },
        {
          "time": 2.0,
          "bones": {
            "spine": { "rotation": [0, 0, 0] },
            "chest": { "rotation": [0, 0, 0] },
            "head": { "rotation": [0, 0, 0] }
          }
        }
      ]
    }
  }
}
```

## Animation Presets

Common animation types and their typical parameters:

| Preset | Duration | Loops | Description |
|--------|----------|-------|-------------|
| Idle | 2.0s | Yes | Subtle breathing/movement |
| Walk | 1.0s | Yes | Walk cycle |
| Run | 0.6s | Yes | Run cycle |
| Jump | 0.8s | No | Jump action |
| Attack | 0.5s | No | Attack action |
| Hit | 0.3s | No | Damage reaction |
| Death | 1.5s | No | Death animation |

## Output Metrics

Generation produces a report with animation metrics:

| Metric | Description |
|--------|-------------|
| `bone_count` | Number of animated bones |
| `animation_frame_count` | Total frame count |
| `animation_duration_seconds` | Animation duration |
| `hinge_axis_violations` | Joints bending wrong way |
| `range_violations` | Rotations exceeding limits |
| `velocity_spikes` | Sudden direction reversals |
| `root_motion_delta` | Root motion [X, Y, Z] |

## Post-Generation Verification

Use `speccade verify` to validate animation metrics:

```bash
speccade verify --report output.report.json --constraints constraints.json
```

Animation constraints include:
- `MaxBoneCount` - Limit bone count
- `MaxHingeAxisViolations` - Limit hinge violations
- `MaxRangeViolations` - Limit range violations
- `MaxVelocitySpikes` - Limit velocity spikes
- `MaxRootMotionDelta` - Limit root motion magnitude

### Example Constraints File

```json
{
  "constraints": [
    { "type": "max_hinge_axis_violations", "value": 0 },
    { "type": "max_range_violations", "value": 5 },
    { "type": "max_velocity_spikes", "value": 3 },
    { "type": "max_root_motion_delta", "value": 10.0 }
  ]
}
```

## Animation Tips

### Looping Animations

For seamless loops, ensure the first and last keyframes have identical bone transforms:

```json
"keyframes": [
  { "time": 0.0, "bones": { "spine": { "rotation": [0, 0, 0] } } },
  { "time": 1.0, "bones": { "spine": { "rotation": [5, 0, 0] } } },
  { "time": 2.0, "bones": { "spine": { "rotation": [0, 0, 0] } } }
]
```

### Bezier Interpolation

Use `bezier` interpolation for organic, smooth motion. Use `linear` for mechanical motion. Use `constant` for snapping effects.

### Bone Naming

Always use bone names from the skeleton preset. For `humanoid_basic_v1`:
- Spine chain: `spine`, `chest`, `neck`, `head`
- Arms: `shoulder_l/r`, `upper_arm_l/r`, `lower_arm_l/r`, `hand_l/r`
- Legs: `upper_leg_l/r`, `lower_leg_l/r`, `foot_l/r`

---

## blender_rigged_v1: IK-Enabled Animation

The `skeletal_animation.blender_rigged_v1` recipe creates animations with IK targets, poses, phases, and procedural layers.

### Main Parameters

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `skeleton_preset` | string | No | Skeleton rig preset (or use `input_armature`/`character`) |
| `clip_name` | string | Yes | Name of the animation clip |
| `duration_frames` | u32 | Yes | Animation duration in frames |
| `duration_seconds` | f64 | No | Alternative: duration in seconds |
| `fps` | u8 | No | Frames per second (default: 30) |
| `loop` | bool | No | Whether animation loops (default: false) |
| `ground_offset` | f64 | No | Root bone Z offset for ground contact |
| `rig_setup` | object | No | IK chains, constraints, foot systems |
| `poses` | object | No | Named pose definitions |
| `phases` | array | No | Animation phases with timing and IK targets |
| `procedural_layers` | array | No | Procedural animation layers |
| `keyframes` | array | No | FK keyframe definitions |
| `ik_keyframes` | array | No | IK target keyframes |
| `interpolation` | string | No | Interpolation mode (default: `linear`) |
| `export` | object | No | Export settings |
| `animator_rig` | object | No | Visual aids for animators |
| `save_blend` | bool | No | Save .blend file alongside GLB |
| `conventions` | object | No | Validation conventions config |

### Rig Setup

Configure IK chains, constraints, and foot systems:

```json
"rig_setup": {
  "presets": ["humanoid_legs", "humanoid_arms"],
  "ik_chains": [],
  "constraints": { "constraints": [] },
  "foot_systems": [],
  "aim_constraints": [],
  "twist_bones": []
}
```

#### IK Presets

| Value | Description |
|-------|-------------|
| `humanoid_legs` | Leg IK with foot targets and knee poles |
| `humanoid_arms` | Arm IK with hand targets and elbow poles |
| `quadruped_forelegs` | Front leg IK for quadrupeds |
| `quadruped_hindlegs` | Back leg IK for quadrupeds |
| `tentacle` | Multi-bone chain without pole |
| `tail` | Tail IK chain |

### Named Poses

Define reusable poses that can be referenced by phases:

```json
"poses": {
  "contact_left": {
    "bones": {
      "spine": { "pitch": 2.0, "yaw": 0.0, "roll": -3.0 },
      "upper_arm_l": { "pitch": 15.0, "yaw": 0.0, "roll": 0.0 }
    }
  }
}
```

### Animation Phases

Break the animation into named phases with IK targets:

```json
"phases": [
  {
    "name": "contact_left",
    "start_frame": 0,
    "end_frame": 15,
    "curve": "ease_in_out",
    "pose": "contact_left",
    "ik_targets": {
      "ik_leg_l": [
        { "frame": 0, "location": [0.1, 0.3, 0.0] },
        { "frame": 15, "location": [0.1, -0.1, 0.0] }
      ]
    }
  }
]
```

#### Phase Parameters

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | string | No | Phase name |
| `start_frame` | i32 | Yes | Start frame |
| `end_frame` | i32 | Yes | End frame |
| `curve` | string | No | Timing curve (`linear`, `ease_in`, `ease_out`, `ease_in_out`) |
| `pose` | string | No | Named pose to apply |
| `ik_targets` | object | No | IK target keyframes per chain |
| `ikfk_blend` | object | No | IK/FK blend keyframes per chain |

### IK Target Keyframes

Animate IK targets directly:

```json
"ik_keyframes": [
  {
    "time": 0.5,
    "targets": {
      "ik_leg_l": {
        "position": [0.1, 0.0, 0.0],
        "rotation": [0, 0, 0],
        "ik_fk_blend": 1.0
      }
    }
  }
]
```

### Example Spec (IK Walk Cycle)

```json
{
  "spec_version": 1,
  "asset_id": "walk_cycle_ik",
  "asset_type": "skeletal_animation",
  "license": "CC0-1.0",
  "seed": 8010,
  "description": "Walk cycle with IK foot targets",
  "outputs": [
    {
      "kind": "primary",
      "format": "glb",
      "path": "walk_cycle_ik.glb"
    }
  ],
  "recipe": {
    "kind": "skeletal_animation.blender_rigged_v1",
    "params": {
      "skeleton_preset": "humanoid_basic_v1",
      "clip_name": "walk_cycle_ik",
      "duration_frames": 60,
      "fps": 30,
      "loop": true,
      "rig_setup": {
        "presets": ["humanoid_legs"]
      },
      "phases": [
        {
          "name": "contact_left",
          "start_frame": 0,
          "end_frame": 30,
          "curve": "ease_in_out",
          "ik_targets": {
            "ik_leg_l": [
              { "frame": 0, "location": [0.1, 0.3, 0.0] },
              { "frame": 30, "location": [0.1, -0.1, 0.0] }
            ]
          }
        }
      ]
    }
  }
}
```

---

## See Also

- [Static Mesh Specs](mesh.md) - Non-deforming meshes
- [Character Specs](character.md) - Skeletal meshes with armatures
