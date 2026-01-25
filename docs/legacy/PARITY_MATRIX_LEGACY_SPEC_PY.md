# Legacy `.spec.py` Key Taxonomy (for `speccade migrate --audit`)

This file is a **legacy-only** inventory of keys seen in the historical Studio `.spec.py` specs
(as represented by the repo’s `golden/legacy/**.spec.py` corpus).

It is used by `speccade migrate --audit` to classify keys as **known** vs **unknown**.

> Note: This does **not** mean the key is automatically mapped into modern SpecCade recipe params.
> Modern generator parity is tracked in `PARITY_MATRIX.md`.

---

## SOUND (audio_sfx)

### Top-Level Keys

| Key | Required | Notes | Status |
|-----|----------|-------|--------|
| `name` | Yes | Used for output filename | ✓ |
| `duration` | No | Total duration in seconds | ✓ |
| `sample_rate` | No | Output sample rate | ✓ |
| `layers` | No | Layer specs | ✓ |
| `normalize` | No | Normalize output | ✓ |
| `peak_db` | No | Target peak level | ✓ |

---

## INSTRUMENT (audio_instrument)

### Top-Level Keys

| Key | Required | Notes | Status |
|-----|----------|-------|--------|
| `name` | Yes | Instrument name | ✓ |
| `synthesis` | No | Synthesis definition | ✓ |
| `envelope` | No | ADSR | ✓ |
| `output` | No | Output settings | ✓ |
| `sample_rate` | No | Sample rate | ✓ |
| `base_note` | No | Base pitch | ✓ |

---

## SONG (music)

### Top-Level Keys

| Key | Required | Notes | Status |
|-----|----------|-------|--------|
| `name` | Yes | Song title / output filename | ✓ |
| `title` | No | Optional display title | ✓ |
| `format` | No | `xm` / `it` | ✓ |
| `bpm` | No | Tempo | ✓ |
| `speed` | No | Ticks per row | ✓ |
| `channels` | No | Channel count | ✓ |
| `restart_position` | No | Loop restart position | ✓ |
| `instruments` | No | Instruments list | ✓ |
| `patterns` | No | Pattern dict | ✓ |
| `arrangement` | No | Pattern order | ✓ |
| `automation` | No | Volume/tempo automation | ✓ |
| `it_options` | No | IT-specific options | ✓ |

---

## TEXTURE (texture_2d)

### Top-Level Keys

| Key | Required | Notes | Status |
|-----|----------|-------|--------|
| `name` | Yes | Output filename | ✓ |
| `size` | No | Output size | ✓ |
| `layers` | No | Layered material description | ✓ |
| `palette` | No | Palette controls | ✓ |
| `color_ramp` | No | Gradient/ramp controls | ✓ |

---

## NORMAL (normal_map)

### Top-Level Keys

| Key | Required | Notes | Status |
|-----|----------|-------|--------|
| `name` | Yes | Output filename | ✓ |
| `size` | No | Output size | ✓ |
| `method` | No | Method selector | ✓ |
| `pattern` | No | Pattern definition | ✓ |
| `processing` | No | Post-processing | ✓ |

---

## MESH (static_mesh)

### Top-Level Keys

| Key | Required | Notes | Status |
|-----|----------|-------|--------|
| `name` | Yes | Output filename | ✓ |
| `primitive` | No | Primitive type | ✓ |
| `params` | No | Primitive parameters | ✓ |
| `location` | No | Transform | ✓ |
| `rotation` | No | Transform | ✓ |
| `scale` | No | Transform | ✓ |
| `shade` | No | Shading mode | ✓ |
| `modifiers` | No | Modifier list | ✓ |
| `uv` | No | UV unwrap options | ✓ |
| `export` | No | Export options | ✓ |

---

## SPEC/CHARACTER (skeletal_mesh)

### Top-Level Keys

| Key | Required | Notes | Status |
|-----|----------|-------|--------|
| `name` | Yes | Output filename | ✓ |
| `skeleton` | No | Skeleton definition | ✓ |
| `parts` | No | Mesh parts | ✓ |
| `texturing` | No | Texturing/material hints | ✓ |
| `tri_budget` | No | Triangle budget | ✓ |

---

## ANIMATION (skeletal_animation)

### Top-Level Keys

| Key | Required | Notes | Status | Migration |
|-----|----------|-------|--------|-----------|
| `name` | Yes | Output filename | ✓ | Maps to `clip_name` |
| `character` | No | Target character reference | ✓ | Maps to `skeleton_preset` |
| `rig` | No | Skeleton rig alias | ✓ | Maps to `skeleton_preset` |
| `skeleton` | No | Skeleton alias | ✓ | Maps to `skeleton_preset` |
| `input_armature` | No | Blender armature input | ✓ | Maps to `skeleton_preset` |
| `duration_frames` | No | Duration | ✓ | Computed to `duration_seconds` |
| `fps` | No | Frame rate | ✓ | Direct map |
| `loop` | No | Looping | ✓ | Direct map |
| `ground_offset` | No | Root offset | ✓ | Warning (not migrated) |
| `poses` | No | Key poses | ✓ | Maps to `keyframes` |
| `phases` | No | Phase sequence | ✓ | Orders `keyframes` |
| `ik_targets` | No | IK target positions | ✓ | Maps to `ik_keyframes` |
| `procedural_layers` | No | Procedural layers | ✓ | Warning (not supported) |
| `rig_setup` | No | Rig setup | ✓ | Export settings only |
| `save_blend` | No | Save .blend | ✓ | Maps to `export.save_blend` |

### Pose Keys (per bone)

| Key | Notes | Status | Migration |
|-----|-------|--------|-----------|
| `frame` | Frame number | ✓ | Used for timing |
| `bones` | Bone transforms dict | ✓ | Maps to keyframe bones |
| `pitch` | X rotation (degrees) | ✓ | Maps to `rotation[0]` |
| `yaw` | Y rotation (degrees) | ✓ | Maps to `rotation[1]` |
| `roll` | Z rotation (degrees) | ✓ | Maps to `rotation[2]` |
| `rotation` | Euler angles array | ✓ | Direct map |
| `position` | Position offset | ✓ | Direct map |
| `location` | Position alias | ✓ | Maps to `position` |
| `scale` | Scale transform | ✓ | Direct map |

### IK Target Keys

| Key | Notes | Status | Migration |
|-----|-------|--------|-----------|
| `position` | World position | ✓ | Direct map |
| `location` | Position alias | ✓ | Maps to `position` |
| `rotation` | World rotation | ✓ | Direct map |
| `blend` | IK/FK blend (0-1) | ✓ | Maps to `ik_fk_blend` |
| `ik_fk_blend` | IK/FK blend alias | ✓ | Direct map |
| `ikfk` | IK/FK blend alias | ✓ | Maps to `ik_fk_blend` |

### Migration Notes

- **Skeleton presets**: Legacy `rig`, `character`, `skeleton`, and `input_armature` all map to `skeleton_preset`. Known values like "humanoid", "biped" map to `humanoid_basic_v1`. Unknown values trigger a warning.
- **IK target naming**: Legacy IK targets without `ik_` prefix automatically get it added (e.g., `foot_l` becomes `ik_foot_l`).
- **Phase-based timing**: When `phases` array is provided, keyframes are distributed evenly from 0.0 to 1.0 normalized time.
- **Procedural layers**: Not supported in canonical format. Use keyframe animation or IK targets instead.
- **Ground offset**: Not migrated. Apply root motion in animation clip or post-process.

