# Animation (Skeletal Animation) Specs

Skeletal animations are keyframe-based animation clips for skeletal meshes.

| Property | Value |
|----------|-------|
| Asset Type | `skeletal_animation` |
| Recipe Kind | `skeletal_animation.blender_clip_v1` |
| Output Formats | `glb` |
| Determinism | Tier 2 (metric validation) |

## SSOT (Source Of Truth)

- Rust types: `crates/speccade-spec/src/recipe/animation/`
- Golden specs: `golden/speccade/specs/skeletal_animation/`
- CLI validation: `speccade validate --spec file.json`

## Recipe Parameters

The `skeletal_animation.blender_clip_v1` recipe creates animation clips with keyframes.

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

## See Also

- [Static Mesh Specs](mesh.md) - Non-deforming meshes
- [Character Specs](character.md) - Skeletal meshes with armatures
