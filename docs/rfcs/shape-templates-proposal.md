# Shape Templates: High-Level Bone Mesh Authoring for LLMs

**Status:** Draft
**Created:** 2026-02-06
**Author:** Claude (Anthropic)
**Related:** RFC-0008 (LLM-Native Asset Authoring), RFC-0010 (Mesh LLM Verification), `llm-3d-generation-analysis.md`

---

## Abstract

This proposal introduces **shape templates** — a high-level abstraction that compiles to `armature_driven_v1` extrusion steps. Shape templates accept semantic, discrete, or bounded parameters that LLMs can reason about, while producing the same deterministic mesh output as hand-tuned extrusion steps. Existing specs using raw `extrusion_steps` remain unchanged.

---

## Problem Statement

SpecCade's `armature_driven_v1` system produces rigged, animatable skeletal meshes from spec files. The core mesh-shaping mechanism — **extrusion steps** — requires sequences of `{extrude, scale, translate, rotate, tilt, bulge}` tuples where values compound cumulatively. This creates three problems for LLM authoring:

1. **Accumulation**: The visual effect of step N depends on all previous steps. LLMs cannot mentally simulate accumulated transforms.
2. **Continuous optimization**: Finding the right `scale: 1.28` (not 1.20, not 1.35) for a "broad chest" requires iterative visual refinement that LLMs cannot perform.
3. **Quality cliff**: There is no middle ground between "uniform cylinder per bone" (easy, low quality) and "hand-tuned extrusion steps" (hard, high quality).

See `llm-3d-generation-analysis.md` for detailed research context.

---

## Proposed Solution

### Overview

Add an optional `shape` field to `ArmatureDrivenBoneMesh` that accepts a **shape template** — a named shape with semantic parameters. The shape template is compiled into `extrusion_steps` (and optionally `attachments`) before being sent to the Blender backend.

```
Spec authoring:    shape template (semantic params)
       ↓           compile (deterministic)
Internal:          extrusion_steps + attachments
       ↓           existing pipeline (unchanged)
Output:            rigged GLB mesh
```

### Design Principles

1. **Compiles to existing primitives** — shape templates produce `extrusion_steps` and `attachments`, not a new rendering path.
2. **All parameters bounded** — every shape template parameter has a valid range. Any value in range produces reasonable output.
3. **Semantic naming** — parameters use descriptive names (`taper`, `muscle_bulge`, `width_ratio`) not abstract ones (`scale`, `translate`).
4. **Backward compatible** — `extrusion_steps` continues to work. `shape` and `extrusion_steps` are mutually exclusive on the same bone mesh.
5. **Deterministic** — same template + parameters always produces identical extrusion steps.

---

## Shape Template Catalog

### `limb` — Arms and legs

Suitable for any elongated body part that tapers along its length.

| Parameter | Type | Range | Default | Description |
|-----------|------|-------|---------|-------------|
| `taper` | float | 0.0–1.0 | 0.3 | How much the limb narrows toward the end. 0 = uniform cylinder, 1 = strong taper |
| `muscle_bulge` | float | 0.0–1.0 | 0.0 | Bulge near the start (bicep/thigh effect). 0 = none, 1 = pronounced |
| `muscle_position` | float | 0.1–0.5 | 0.25 | Where along the bone the muscle bulge peaks |
| `segments` | int | 2–6 | 3 | Number of extrusion steps generated |

**Compilation example** (`taper: 0.4, muscle_bulge: 0.5, segments: 3`):
```python
# Compiles to:
"extrusion_steps": [
    {"extrude": 0.25, "scale": 1.15},   # Muscle bulge region
    {"extrude": 0.50, "scale": 0.82},   # Mid-limb taper
    {"extrude": 0.25, "scale": 0.70},   # End taper
]
```

### `torso` — Chest, spine, and hip regions

Handles the complex shape of trunk segments with width/depth differentiation.

| Parameter | Type | Range | Default | Description |
|-----------|------|-------|---------|-------------|
| `width_ratio` | float | 0.5–2.0 | 1.0 | Width relative to depth. >1 = wide/flat (broad chest), <1 = deep/narrow |
| `taper_start` | float | -0.5–0.5 | 0.0 | Taper at the bottom. Negative = flare out, positive = narrow |
| `taper_end` | float | -0.5–0.5 | 0.0 | Taper at the top. Positive = narrow toward neck |
| `belly` | float | 0.0–1.0 | 0.0 | Mid-section outward bulge |
| `segments` | int | 2–6 | 3 | Number of extrusion steps generated |

**Compilation example** (`width_ratio: 1.3, taper_end: 0.4, belly: 0.0, segments: 3`):
```python
# Compiles to:
"profile_radius": [base_r * 1.14, base_r * 0.88],  # elliptical for width_ratio
"extrusion_steps": [
    {"extrude": 0.20, "scale": 1.05},   # Base
    {"extrude": 0.50, "scale": 1.10},   # Main body
    {"extrude": 0.30, "scale": 0.60},   # Taper to top
]
```

### `sphere` — Head, joints, rounded ends

For body parts that approximate a sphere or ellipsoid.

| Parameter | Type | Range | Default | Description |
|-----------|------|-------|---------|-------------|
| `squash` | float | 0.5–2.0 | 1.0 | Vertical squash/stretch. <1 = oblate (pumpkin), >1 = prolate (egg) |
| `jaw` | float | 0.0–1.0 | 0.0 | Lower narrowing (chin/jaw effect). 0 = symmetric sphere |
| `brow` | float | 0.0–1.0 | 0.0 | Upper protrusion (brow ridge). 0 = smooth top |
| `segments` | int | 3–8 | 4 | Number of extrusion steps (more = smoother sphere) |

**Compilation example** (`squash: 1.0, jaw: 0.3, segments: 4`):
```python
# Compiles to:
"extrusion_steps": [
    {"extrude": 0.10, "scale": 1.40},   # Narrow jaw
    {"extrude": 0.25, "scale": 1.80},   # Widen to cheeks
    {"extrude": 0.40, "scale": 1.10},   # Cranium
    {"extrude": 0.25, "scale": 0.45},   # Crown taper
]
```

### `box` — Blocky body parts, robot segments

For rectangular/boxy shapes.

| Parameter | Type | Range | Default | Description |
|-----------|------|-------|---------|-------------|
| `bevel` | float | 0.0–1.0 | 0.1 | Edge rounding. 0 = sharp edges, 1 = very rounded (approaches cylinder) |
| `taper` | float | 0.0–1.0 | 0.0 | Taper toward end. 0 = uniform box |
| `width_ratio` | float | 0.5–2.0 | 1.0 | Width vs depth ratio |

**Compilation example** (`bevel: 0.2, taper: 0.0, width_ratio: 1.5`):
```python
# Compiles to:
"profile": "square",
"profile_radius": [base_r * 1.22, base_r * 0.82],  # rectangular for width_ratio
"extrusion_steps": [0.5, 0.5],  # Uniform box
"modifiers": [{"bevel": {"width": 0.02, "segments": 2}}],
```

### `cone` — Pointed shapes, horns, tails

Tapers to a point or near-point.

| Parameter | Type | Range | Default | Description |
|-----------|------|-------|---------|-------------|
| `point_sharpness` | float | 0.0–1.0 | 0.8 | How pointed the tip is. 0 = blunt cone, 1 = sharp point |
| `curve` | float | -1.0–1.0 | 0.0 | Curvature of the taper. 0 = linear, >0 = convex (horn-like), <0 = concave |
| `segments` | int | 2–6 | 3 | Number of extrusion steps |

### `wedge` — Feet, hands, flattened extremities

For shapes that transition from round/thick to flat/wide.

| Parameter | Type | Range | Default | Description |
|-----------|------|-------|---------|-------------|
| `splay` | float | 0.0–1.0 | 0.5 | How much the shape fans out. 0 = stays round, 1 = very flat/wide |
| `thickness` | float | 0.2–1.0 | 0.5 | Vertical thickness at the end. 0.2 = very flat, 1.0 = round |
| `taper_tip` | float | 0.0–1.0 | 0.3 | Narrowing at the very tip (fingers/toes) |
| `segments` | int | 3–6 | 4 | Number of extrusion steps |

**Compilation example** (`splay: 0.6, thickness: 0.4, taper_tip: 0.5, segments: 4`):
```python
# Compiles to:
"extrusion_steps": [
    {"extrude": 0.15, "scale": 1.30},               # Wrist/ankle transition
    {"extrude": 0.30, "scale": [1.50, 0.60]},        # Fan out, flatten
    {"extrude": 0.35, "scale": [1.10, 0.85]},        # Palm/sole body
    {"extrude": 0.20, "scale": [0.40, 0.50]},        # Finger/toe taper
]
```

---

## Concrete Before/After Examples

### Example 1: Muscular Character

**Before** (raw extrusion steps — 48 lines for upper arm alone):
```python
"upper_arm_l": {
    "profile": "circle(10)",
    "profile_radius": {"absolute": 0.065},
    "extrusion_steps": [
        {"extrude": 0.15, "scale": 1.25},
        {"extrude": 0.25, "scale": 1.18},
        {"extrude": 0.40, "scale": 0.85},
        {"extrude": 0.20, "scale": 0.72},
    ],
    "attachments": [
        {"primitive": "sphere", "dimensions": [0.04, 0.05, 0.08],
         "offset": [0.0, 0.04, -0.08], "material_index": 1},
    ],
},
```

**After** (shape template — 6 lines):
```python
"upper_arm_l": {
    "profile": "circle(10)",
    "profile_radius": {"absolute": 0.065},
    "shape": {"limb": {"taper": 0.45, "muscle_bulge": 0.7, "muscle_position": 0.25}},
},
```

The template compiles to equivalent extrusion steps and can optionally generate the bicep attachment sphere automatically when `muscle_bulge` exceeds a threshold.

### Example 2: Simple Robot Character

**Before** (trying to achieve boxy look with circle profiles):
```python
"chest": {
    "profile": "circle(8)",
    "profile_radius": {"absolute": 0.15},
    "extrusion_steps": [0.5, 0.5],
    "modifiers": [{"bevel": {"width": 0.01, "segments": 1}}],
},
```

**After** (explicit box intent):
```python
"chest": {
    "profile_radius": {"absolute": 0.15},
    "shape": {"box": {"bevel": 0.1, "width_ratio": 1.3}},
},
```

The `box` template sets the profile to `square`, applies bevel modifiers, and handles width/depth ratio via elliptical profile radius.

### Example 3: Character with a Wizard Hat

Using an extra bone for accessories:

```python
# Skeleton includes a hat bone extending upward from head
{"bone": "hat", "head": [0, 0, 1.02], "tail": [0, 0, 1.35], "parent": "head"},

# Hat bone mesh uses cone template
"hat": {
    "profile": "circle(12)",
    "profile_radius": {"absolute": 0.14},
    "shape": {"cone": {"point_sharpness": 0.7, "curve": 0.3}},
    "cap_start": True,
    "cap_end": True,
    "material_index": 1,  # hat_fabric material
},
```

### Example 4: Character with a Skirt

Using extra bones radiating downward from the hips:

```python
# Skeleton includes skirt bones (one per panel, or simpler: front/back/sides)
{"bone": "skirt_front", "head": [0, 0.08, 0.10], "tail": [0, 0.12, -0.20], "parent": "hips"},
{"bone": "skirt_back",  "head": [0, -0.08, 0.10], "tail": [0, -0.12, -0.20], "parent": "hips"},
{"bone": "skirt_l",     "head": [0.08, 0, 0.10], "tail": [0.15, 0, -0.20], "parent": "hips"},
{"bone": "skirt_r",     "mirror": "skirt_l"},

# Each skirt panel uses a wedge template for the flared shape
"skirt_front": {
    "profile": "circle(8)",
    "profile_radius": {"absolute": 0.08},
    "shape": {"cone": {"point_sharpness": 0.0, "curve": -0.3}},
    "cap_start": True,
    "cap_end": True,
},
```

### Example 5: Hair

Using extra bones for hair strands or clumps:

```python
# Hair bones cascade from top/sides of head
{"bone": "hair_top",  "head": [0, -0.02, 1.08], "tail": [0, -0.10, 0.90], "parent": "head"},
{"bone": "hair_l",    "head": [0.08, 0, 1.00], "tail": [0.12, 0, 0.75], "parent": "head"},
{"bone": "hair_r",    "mirror": "hair_l"},

# Hair uses cone template with curve for flowing shape
"hair_top": {
    "profile": "circle(6)",
    "profile_radius": {"absolute": 0.06},
    "shape": {"cone": {"point_sharpness": 0.4, "curve": 0.2}},
    "material_index": 2,  # hair material
},
```

---

## Spec Contract Changes

### Rust Type Additions

The following types would be added to `crates/speccade-spec/src/recipe/character/armature_driven.rs`:

```rust
/// Shape template — compiles to extrusion_steps before mesh generation.
/// Mutually exclusive with `extrusion_steps` (validation error if both set).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum ShapeTemplate {
    /// Elongated body part (arms, legs, fingers, tails).
    #[serde(rename = "limb")]
    Limb(LimbShape),
    /// Trunk segment (chest, spine, hips).
    #[serde(rename = "torso")]
    Torso(TorsoShape),
    /// Spherical/ellipsoidal (head, joints).
    #[serde(rename = "sphere")]
    Sphere(SphereShape),
    /// Rectangular/boxy (robot parts, crates).
    #[serde(rename = "box")]
    Box(BoxShape),
    /// Pointed taper (horns, hats, tails).
    #[serde(rename = "cone")]
    Cone(ConeShape),
    /// Flat/splayed extremity (hands, feet, fins).
    #[serde(rename = "wedge")]
    Wedge(WedgeShape),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LimbShape {
    /// Taper from start to end. 0.0 = uniform, 1.0 = strong taper.
    #[serde(default = "default_limb_taper")]
    pub taper: f64,           // 0.0..=1.0

    /// Muscle bulge intensity near the start. 0.0 = none, 1.0 = pronounced.
    #[serde(default)]
    pub muscle_bulge: f64,    // 0.0..=1.0

    /// Position of muscle bulge peak along the bone. 0.1 = near start, 0.5 = mid.
    #[serde(default = "default_muscle_position")]
    pub muscle_position: f64, // 0.1..=0.5

    /// Number of extrusion steps to generate.
    #[serde(default = "default_limb_segments")]
    pub segments: u8,         // 2..=6
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TorsoShape {
    /// Width-to-depth ratio. >1 = wide/flat, <1 = deep/narrow.
    #[serde(default = "default_one")]
    pub width_ratio: f64,     // 0.5..=2.0

    /// Taper at bottom. Negative = flare, positive = narrow.
    #[serde(default)]
    pub taper_start: f64,     // -0.5..=0.5

    /// Taper at top. Positive = narrow (toward neck).
    #[serde(default)]
    pub taper_end: f64,       // -0.5..=0.5

    /// Mid-section outward bulge (belly).
    #[serde(default)]
    pub belly: f64,           // 0.0..=1.0

    #[serde(default = "default_torso_segments")]
    pub segments: u8,         // 2..=6
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SphereShape {
    /// Vertical squash/stretch. <1 = oblate, >1 = prolate.
    #[serde(default = "default_one")]
    pub squash: f64,          // 0.5..=2.0

    /// Lower narrowing (jaw/chin). 0 = symmetric.
    #[serde(default)]
    pub jaw: f64,             // 0.0..=1.0

    /// Upper protrusion (brow ridge). 0 = smooth.
    #[serde(default)]
    pub brow: f64,            // 0.0..=1.0

    #[serde(default = "default_sphere_segments")]
    pub segments: u8,         // 3..=8
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BoxShape {
    /// Edge rounding. 0 = sharp, 1 = fully rounded.
    #[serde(default = "default_box_bevel")]
    pub bevel: f64,           // 0.0..=1.0

    /// Taper toward end. 0 = uniform box.
    #[serde(default)]
    pub taper: f64,           // 0.0..=1.0

    /// Width-to-depth ratio.
    #[serde(default = "default_one")]
    pub width_ratio: f64,     // 0.5..=2.0
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConeShape {
    /// Tip sharpness. 0 = blunt, 1 = sharp point.
    #[serde(default = "default_cone_sharpness")]
    pub point_sharpness: f64, // 0.0..=1.0

    /// Curvature. 0 = linear, >0 = convex (horn), <0 = concave.
    #[serde(default)]
    pub curve: f64,           // -1.0..=1.0

    #[serde(default = "default_cone_segments")]
    pub segments: u8,         // 2..=6
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WedgeShape {
    /// Splay amount. 0 = stays round, 1 = very flat/wide.
    #[serde(default = "default_wedge_splay")]
    pub splay: f64,           // 0.0..=1.0

    /// Vertical thickness at end. 0.2 = very flat, 1.0 = round.
    #[serde(default = "default_wedge_thickness")]
    pub thickness: f64,       // 0.2..=1.0

    /// Narrowing at the tip.
    #[serde(default = "default_wedge_taper")]
    pub taper_tip: f64,       // 0.0..=1.0

    #[serde(default = "default_wedge_segments")]
    pub segments: u8,         // 3..=6
}
```

### `ArmatureDrivenBoneMesh` Change

Add one field:

```rust
pub struct ArmatureDrivenBoneMesh {
    // ... existing fields ...

    /// High-level shape template. Mutually exclusive with `extrusion_steps`.
    /// Compiled to extrusion_steps (and optionally attachments) before generation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shape: Option<ShapeTemplate>,
}
```

### Validation Rules

1. If `shape` is set and `extrusion_steps` is non-empty, emit validation error.
2. All shape template parameters are clamped to their valid ranges during validation (warn, don't reject).
3. `shape` templates inherit `profile` and `profile_radius` from the bone mesh — the template only controls the extrusion geometry.

---

## Implementation Approach

### Option A: Rust Pre-processing (Recommended)

Add a compilation pass in the Rust spec layer that expands shape templates into extrusion steps before the spec reaches the Blender backend.

**Location:** New module `crates/speccade-spec/src/recipe/character/shape_compiler.rs`

**Flow:**
```
Parse spec → Validate → Compile shape templates → Serialize to Blender JSON → Blender backend
```

**Advantages:**
- Blender handler unchanged — it still receives extrusion steps.
- Compilation is testable with unit tests (no Blender needed).
- Determinism is easy to verify.
- Can inspect compiled output for debugging (`speccade compile --show-expanded`).

**Disadvantages:**
- Adds a compilation pass to the pipeline.

### Option B: Blender Python Pre-processing

Add template compilation in `blender/speccade/armature_driven.py` before the extrusion loop.

**Advantages:**
- No Rust changes needed.
- Templates can directly use Blender geometry operations.

**Disadvantages:**
- Harder to test (requires Blender).
- Compilation logic split across Rust (types/validation) and Python (compilation).
- Determinism harder to verify.

### Recommendation

**Option A** (Rust pre-processing). Shape template compilation is pure math (semantic params → extrusion step sequences). It doesn't need Blender's geometry engine and benefits from Rust's testability.

---

## Compilation Algorithm Sketch

For the `limb` template as an example:

```rust
fn compile_limb(shape: &LimbShape, profile_radius: f64) -> Vec<ExtrusionStep> {
    let n = shape.segments as usize;
    let mut steps = Vec::with_capacity(n);

    for i in 0..n {
        let t = i as f64 / (n - 1) as f64;  // 0.0 to 1.0 along bone

        // Base taper: linear interpolation from 1.0 to (1.0 - taper)
        let taper_scale = 1.0 - shape.taper * t;

        // Muscle bulge: gaussian-like bump centered at muscle_position
        let bulge_dist = (t - shape.muscle_position).abs();
        let bulge_width = 0.15;
        let bulge_scale = shape.muscle_bulge * 0.3
            * (-bulge_dist * bulge_dist / (2.0 * bulge_width * bulge_width)).exp();

        // Combine: multiply taper with additive bulge
        let scale = taper_scale + bulge_scale;

        // Even distribution of extrusion distances
        let extrude = 1.0 / n as f64;

        steps.push(ExtrusionStep::Full(ExtrusionStepDef {
            extrude,
            scale: Some(ScaleValue::Uniform(scale)),
            translate: None,
            rotate: None,
            tilt: None,
            bulge: None,
        }));
    }

    steps
}
```

Each template has a similar compilation function. The key property is that all parameters are bounded, so the output is always valid geometry.

---

## Backward Compatibility

- **Existing specs**: No changes. `extrusion_steps` works exactly as before.
- **New specs**: Can use `shape` instead of `extrusion_steps`, or continue using `extrusion_steps` for full control.
- **Migration**: Not required. Shape templates are additive.
- **Interop**: A bone mesh can use `shape` for the base geometry and still add `attachments`, `modifiers`, `translate`, `rotate`, `cap_start`, `cap_end`, and connection modes.

---

## Accessories, Clothing, and Hair

Shape templates handle these through the same bone mechanism:

### Approach: Extra Bones + Shape Templates

Accessories are modeled as **additional bones** in the skeleton with their own bone meshes:

| Accessory | Bone Placement | Shape Template |
|-----------|---------------|----------------|
| Hat/helmet | Extends upward from head top | `cone` or `sphere` (depending on hat type) |
| Skirt/cape | Bones radiate downward from hips/chest | `cone` (flared) or `limb` (tapered) |
| Hair strands | Cascade from head top/sides | `cone` (with `curve` for flow) |
| Tail | Extends backward from root | `limb` (with `taper`) or `cone` |
| Wings | Extend laterally from chest/spine | `wedge` (flat, splayed) |
| Belt/collar | Short bone at waist/neck | `torso` (with `width_ratio` for shape) |
| Weapon holster | Short bone at hip | `box` (rigid attachment) |

This is already supported by the existing skeleton + bone_mesh system — shape templates just make it easier to describe the accessory's form.

### Example: Full Character with Accessories

```python
"skeleton": [
    # Standard humanoid bones...
    {"bone": "root", "head": [0,0,0], "tail": [0,0,0.1]},
    # ... (standard 20 bones) ...

    # Accessories as extra bones
    {"bone": "hat_brim", "head": [0,0,1.02], "tail": [0,0,1.06], "parent": "head"},
    {"bone": "hat_cone", "head": [0,0,1.06], "tail": [0,0,1.35], "parent": "hat_brim"},
    {"bone": "cape_l", "head": [0.12,0,0.68], "tail": [0.15,-0.1,0.10], "parent": "chest"},
    {"bone": "cape_r", "mirror": "cape_l"},
],
"bone_meshes": {
    # Standard body with shape templates...
    "chest": {
        "profile": "circle(12)",
        "profile_radius": {"absolute": 0.13},
        "shape": {"torso": {"width_ratio": 1.3, "taper_end": 0.35}},
    },
    "upper_arm_l": {
        "profile": "circle(10)",
        "profile_radius": {"absolute": 0.06},
        "shape": {"limb": {"taper": 0.35, "muscle_bulge": 0.4}},
    },
    # ... more body parts ...

    # Accessories
    "hat_brim": {
        "profile": "circle(12)",
        "profile_radius": {"absolute": 0.16},
        "shape": {"torso": {"width_ratio": 1.0, "taper_start": -0.3, "taper_end": -0.3}},
        "cap_start": True,
        "material_index": 2,
    },
    "hat_cone": {
        "profile": "circle(10)",
        "profile_radius": {"absolute": 0.10},
        "shape": {"cone": {"point_sharpness": 0.6, "curve": 0.2}},
        "cap_end": True,
        "material_index": 2,
    },
    "cape_l": {
        "profile": "circle(6)",
        "profile_radius": {"absolute": 0.08},
        "shape": {"cone": {"point_sharpness": 0.1, "curve": -0.2}},
        "material_index": 3,
    },
    "cape_r": {"mirror": "cape_l"},
},
```

---

## Character Presets with Shape Templates

Shape templates enable higher-level character presets that combine skeleton layout with default shapes:

```python
# Hypothetical future stdlib function
def muscular_humanoid(height=1.0, build="athletic"):
    """Returns skeleton + bone_meshes using shape templates."""
    return {
        "skeleton_preset": "humanoid_connected_v1",
        "bone_meshes": {
            "chest": {"shape": {"torso": {"width_ratio": 1.4, "taper_end": 0.4}}},
            "upper_arm_l": {"shape": {"limb": {"taper": 0.4, "muscle_bulge": 0.6}}},
            "head": {"shape": {"sphere": {"jaw": 0.3}}},
            # ... etc
        }
    }
```

This is out of scope for the initial implementation but illustrates the composability.

---

## Testing Strategy

1. **Unit tests** in `shape_compiler.rs`:
   - Each template produces valid extrusion steps (distances sum to ~1.0, scales > 0).
   - Boundary values (all params at min, all at max) produce valid output.
   - Default parameters produce reasonable output.

2. **Golden tests** as `.star` specs:
   - `specs/character/shape_template_limb.star` — exercises limb template variants.
   - `specs/character/shape_template_torso.star` — exercises torso template variants.
   - `specs/character/shape_template_full_character.star` — complete character using only templates.

3. **Round-trip verification**:
   - Compile template → generate mesh → verify triangle count within expected range.
   - Compare template-generated mesh metrics against hand-tuned equivalents.

---

## Open Questions

1. **Should templates generate attachments?** For instance, should `limb` with `muscle_bulge > 0.5` automatically add a sphere attachment for muscle definition? This increases convenience but reduces control.

2. **Profile selection**: Should templates set the `profile` automatically (e.g., `box` always uses `square`)? Or should the author still specify profile separately?

3. **Composition**: Should multiple shape templates be composable on a single bone? E.g., a `limb` base with a `cone` tip? Current proposal says no — one shape per bone mesh, use separate bones for complex shapes.

4. **Parameter interpolation for animation**: If a character has a "flex" animation, should shape template parameters be animatable? This would require runtime compilation, adding significant complexity.

---

## Rollout Plan

### Phase 1: Core Templates (MVP)
- Add `ShapeTemplate` enum and struct types to `speccade-spec`.
- Implement `compile_shape()` for `limb`, `torso`, and `sphere` templates.
- Add validation (mutual exclusivity with `extrusion_steps`, parameter bounds).
- Add 3 golden test specs.
- Update `docs/spec-reference/character.md`.

### Phase 2: Full Catalog
- Add `box`, `cone`, `wedge` templates.
- Add more golden tests with accessories (hat, cape, hair).
- Document patterns for common character archetypes.

### Phase 3: LLM Authoring Integration
- Add shape template examples to LLM authoring references.
- Add `shape_template` parameter hints to JSON schema.
- Create a library of character archetypes using templates (warrior, wizard, robot, animal).
- Integrate with RFC-0010 verification feedback for template parameter suggestions.
