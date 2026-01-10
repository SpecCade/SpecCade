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

| Key | Required | Notes | Status |
|-----|----------|-------|--------|
| `name` | Yes | Output filename | ✓ |
| `character` | No | Target character reference | ✓ |
| `input_armature` | No | Blender armature input | ✓ |
| `duration_frames` | No | Duration | ✓ |
| `fps` | No | Frame rate | ✓ |
| `loop` | No | Looping | ✓ |
| `ground_offset` | No | Root offset | ✓ |
| `poses` | No | Key poses | ✓ |
| `phases` | No | Phase sequence | ✓ |
| `procedural_layers` | No | Procedural layers | ✓ |
| `rig_setup` | No | Rig setup | ✓ |
| `save_blend` | No | Save .blend | ✓ |

