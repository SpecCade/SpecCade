# Animation Migration Test Fixtures

These fixtures test the migration of legacy `.spec.py` ANIMATION dicts to canonical `skeletal_animation.blender_clip_v1` specs.

## Fixtures

### walk_cycle.spec.py
- **Tests**: Basic walk cycle with phases, poses, and bone rotations
- **Features**: `phases` ordering, `poses` with `bones`, `loop: true`
- **Expected**: 4 keyframes (contact_l, passing_r, contact_r, passing_l)

### run_cycle.spec.py
- **Tests**: Faster animation timing with position transforms
- **Features**: `character` reference, position on hips bone, `loop: true`
- **Expected**: 4 keyframes with root motion

### idle_breathe.spec.py
- **Tests**: Subtle breathing animation with scale transforms
- **Features**: `skeleton` reference, `scale` transforms on chest, longer duration
- **Expected**: 4 keyframes with scale data preserved

### punch_attack.spec.py
- **Tests**: Non-looping attack animation with yaw rotations
- **Features**: `rig` reference, multi-axis rotation (pitch/yaw/roll), `loop: false`, 60 fps
- **Expected**: 4 keyframes with full rotation data

### jump.spec.py
- **Tests**: Jump with IK targets and position transforms
- **Features**: `input_armature` reference, `ik_targets`, `ground_offset`, `save_blend`, position transforms
- **Expected**: 4 keyframes + IK keyframes, warnings for ground_offset

## Legacy Key Mapping

| Legacy Key | Canonical Path | Notes |
|------------|----------------|-------|
| `name` | `clip_name` | Animation name |
| `rig`/`character`/`skeleton`/`input_armature` | `skeleton_preset` | Skeleton reference |
| `fps` | `fps` | Direct map |
| `duration_frames` | Computed to `duration_seconds` | Using fps |
| `loop` | `loop` | Direct map |
| `poses` | `keyframes` | With bone transforms |
| `phases` | Keyframe ordering | Distributes time evenly |
| `ik_targets` | `ik_keyframes` | With normalized names |
| `ground_offset` | Warning | Not migrated |
| `procedural_layers` | Warning | Not supported |
| `save_blend` | `export.save_blend` | Export setting |

## Running Tests

```bash
cargo test -p speccade-cli -- animation --nocapture
cargo test -p speccade-cli migrate
```
