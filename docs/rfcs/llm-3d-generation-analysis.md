# LLM 3D Generation: Research Summary and Gap Analysis

**Status:** Draft
**Created:** 2026-02-06
**Author:** Claude (Anthropic)
**Related:** RFC-0008 (LLM-Native Asset Authoring), RFC-0010 (Mesh LLM Verification)

---

## Abstract

This document surveys the current state of LLM-driven 3D content generation, identifies why primitive-composition approaches (Three.js, CSG) produce better perceived output than parametric extrusion systems, and analyzes the specific gap between SpecCade's `armature_driven_v1` interface and what LLMs need to generate quality character meshes. The conclusion is that SpecCade's skeleton/armature infrastructure is sound, but the extrusion step parameterization creates a continuous numerical optimization problem that LLMs are poorly suited to solve.

---

## 1. LLM Capabilities for 3D Generation

### 1.1 What LLMs Can Do

Research and practical experience show LLMs are effective at:

- **Compositional reasoning**: Decomposing objects into named parts ("a character has a torso, two arms, two legs, a head") and describing their spatial relationships ("arms attach at shoulder height, legs extend downward from hips").
- **Semantic description**: Expressing shape intent in natural language ("muscular upper arm", "tapered forearm", "wide hips").
- **Code generation with familiar APIs**: Producing Three.js, OpenSCAD, or similar code where each line maps to a visible geometric operation (e.g., `new THREE.SphereGeometry(0.5)`, `cylinder(r=0.3, h=1.0)`).
- **Discrete parameter selection**: Choosing from enumerated options ("body_type: athletic", "head_shape: round") reliably.
- **Relative proportions**: Reasoning about ratios ("shoulders wider than hips", "arms reach to mid-thigh").

### 1.2 What LLMs Cannot Do

- **Precise continuous parameterization**: When given 50+ interdependent floating-point values, LLMs cannot predict their combined visual effect. A `scale: 1.28` vs `scale: 1.15` at extrusion step 2 of 4 produces a subtly different silhouette that the LLM has no perceptual model to evaluate.
- **Multi-step accumulation**: Extrusion steps compound — the visual result of step 3 depends on the exact values chosen at steps 1 and 2. LLMs reason about each step locally but cannot mentally simulate the accumulated 3D geometry.
- **Coordinate system spatial reasoning**: Mapping between semantic intent ("bicep bulge on the front of the arm") and specific `[x, y, z]` offsets in a bone-local coordinate system requires mental rotation that LLMs perform unreliably.
- **Iterative numerical refinement**: Without visual feedback, adjusting `profile_radius: 0.065` to `0.068` is random walk, not informed editing.

### 1.3 Evidence from Research

The LLM 3D generation landscape divides into two paradigms:

**Parametric generation** (what SpecCade currently requires):
- ShapeNet/PartNet studies show LLMs can describe part decompositions but struggle to produce precise mesh parameters.
- Text-to-CAD systems (e.g., CAD-as-Language) achieve reasonable results only with heavily constrained parameter spaces (discrete feature catalogs, not continuous extrusion).
- Chain-of-thought prompting improves discrete decisions but does not help with continuous parameter tuning.

**Compositional/primitive generation** (what Three.js demos use):
- Systems like 3D-GPT and SceneScript demonstrate that LLMs excel at placing and sizing primitive shapes.
- Each primitive is independently parameterized (position, size, rotation) — no accumulation effects.
- The mapping from intent to code is direct: "add a sphere at [0, 1, 0] with radius 0.3" is one line of code with a visible result.

The quality gap between these approaches is not about rendering fidelity — it's about authoring fidelity. An LLM writing Three.js can predict what it's creating; an LLM writing extrusion steps cannot.

---

## 2. Why Three.js / Primitive Composition Wins (Perceived Quality)

### 2.1 Direct Intent-to-Geometry Mapping

In Three.js or CSG-based generation, every line of code produces a predictable geometric primitive:

```javascript
// LLM can reason about each line independently
const torso = new THREE.BoxGeometry(0.4, 0.6, 0.3);     // wide, tall, shallow box
const head = new THREE.SphereGeometry(0.15);              // round head
const arm = new THREE.CylinderGeometry(0.04, 0.03, 0.5); // tapered cylinder
```

Each primitive has immediate semantic meaning. The LLM knows that `BoxGeometry(0.4, 0.6, 0.3)` produces a shape that is wider than it is deep. There is no accumulation — each shape exists independently in world space.

### 2.2 No Accumulation Problem

With extrusion steps, the geometry at any point depends on all previous steps:

```python
# What does this look like? The LLM cannot mentally simulate.
"extrusion_steps": [
    {"extrude": 0.15, "scale": 1.28},   # Step 1: widens
    {"extrude": 0.55, "scale": 1.15},   # Step 2: widens more (from already-widened)
    {"extrude": 0.30, "scale": 0.55},   # Step 3: tapers (from double-widened)
]
```

The final geometry is the product of all scale factors applied cumulatively to the initial profile radius. The LLM would need to compute `0.13 * 1.28 * 1.15 * 0.55` to know the final radius at the top of the chest — and that's before considering bone-relative vs absolute units, tilt, bulge, and translate interactions.

### 2.3 Forgiving Error Characteristics

Three.js/primitive errors degrade gracefully:
- A sphere slightly misplaced still looks like a sphere.
- A cylinder with wrong length still reads as a limb.
- Overlapping primitives still convey the intended shape.

Extrusion step errors can catastrophically deform the mesh:
- A single `scale: 2.05` instead of `scale: 1.05` produces a bizarre balloon shape.
- A `tilt` on the wrong step can twist the entire remainder of the bone mesh.
- Steps that don't sum to ~1.0 produce geometry that extends past or falls short of the bone.

### 2.4 Semantic Density

Primitive code is semantically dense — a reader (human or LLM) can understand the intent from the code:

```javascript
// "Add pectoral muscles as flattened spheres on the front of the chest"
const pec_l = new THREE.SphereGeometry(0.08, 0.065, 0.055);
pec_l.position.set(0.055, 0.08, 0.08);  // left, forward, up
```

Extrusion step code encodes shape implicitly through accumulated transforms — the intent must be inferred from comments, not code:

```python
{"extrude": 0.15, "scale": 1.28},   # What is this? Only "Wide chest" comment explains
{"extrude": 0.55, "scale": 1.15},   # Why 1.15? Because 1.28 * 1.15 ≈ desired width
```

---

## 3. What SpecCade Currently Provides

### 3.1 The Armature-Driven Architecture

SpecCade's `skeletal_mesh.armature_driven_v1` recipe provides:

| Component | What It Does | LLM-Friendliness |
|-----------|-------------|-------------------|
| **Custom skeleton** | Array of bones with explicit `[x,y,z]` head/tail positions | Medium — bones are compositional but coordinates require spatial reasoning |
| **Skeleton presets** | Named rigs (e.g., `humanoid_connected_v1`) | High — one string replaces 20+ bone definitions |
| **Bone meshes** | Per-bone mesh definitions with profile + extrusion steps | Low — the core problem area |
| **Mirror references** | `{"mirror": "arm_upper_l"}` copies + flips a mesh definition | High — halves the authoring effort |
| **Attachments** | Primitive shapes (sphere, cube, cone, cylinder) attached to bones | High — direct primitive composition |
| **Material slots** | Named materials with PBR properties | Medium — color values are intuitive but roughness/metallic less so |
| **Modifiers** | Bevel, subdivide, boolean operations | Medium — discrete operations with few parameters |
| **Bridge connections** | Topological mesh connections between parent/child bones | Low — requires matching topology and coaxial bones |

### 3.2 The Complexity Spectrum

SpecCade specs range from simple to complex. Consider two real specs from the repository:

**`simple_biped.star`** — 62 lines, ~800 triangles:
```python
# Easy for LLMs: each bone mesh is one line with 2-3 parameters
"chest": {"profile": "circle(8)", "profile_radius": 0.14},
"neck":  {"profile": "circle(8)", "profile_radius": 0.05},
"head":  {"profile": "circle(10)", "profile_radius": {"absolute": 0.09}, "cap_end": True},
```

This is essentially primitive composition — each bone gets a uniform cylinder. An LLM can author this reliably. But the result is a featureless mannequin with no anatomical detail.

**`humanoid_male_v2.star`** — 360 lines, ~20,000 triangles:
```python
# Hard for LLMs: multi-step extrusions with accumulated transforms
"chest": {
    "profile": "hexagon(8)",
    "profile_radius": {"absolute": 0.13},
    "extrusion_steps": [
        {"extrude": 0.15, "scale": 1.28},   # Wide chest
        {"extrude": 0.55, "scale": 1.15},   # Main chest
        {"extrude": 0.30, "scale": 0.55},   # Taper to neck
    ],
    "attachments": [
        {"primitive": "sphere", "dimensions": [0.08, 0.065, 0.055],
         "offset": [0.055, 0.08, 0.08], "material_index": 1},  # Left pectoral
        {"primitive": "sphere", "dimensions": [0.08, 0.065, 0.055],
         "offset": [-0.055, 0.08, 0.08], "material_index": 1}, # Right pectoral
        {"primitive": "cylinder", "dimensions": [0.015, 0.015, 0.08],
         "offset": [0.0, 0.05, 0.08], "rotation": [90.0, 0.0, 0.0],
         "material_index": 0},  # Sternum groove
    ],
}
```

This produces recognizable anatomy, but requires an author who understands:
1. How `profile_radius * scale_1 * scale_2 * ...` accumulates
2. Bone-local coordinate systems (which direction is "forward" on the chest bone?)
3. How extrusion fractions relate to physical proportions
4. How attachment offsets interact with the extruded mesh shape

### 3.3 What LLMs Actually Produce

When asked to write an `armature_driven_v1` spec from scratch, LLMs typically:

1. **Produce valid syntax** — Starlark parsing succeeds.
2. **Choose reasonable bone positions** — The skeleton layout is plausible.
3. **Set profile radii in the right ballpark** — Arms thinner than torso, head roughly right.
4. **Fail at extrusion step tuning** — Scale values are guesses that produce lumpy, inconsistent silhouettes.
5. **Misplace attachments** — Offset coordinates put features in wrong locations because the LLM doesn't know the bone-local coordinate system orientation for non-vertical bones.
6. **Produce better results with simple specs** — When told to skip extrusion steps and just use uniform profiles (like `simple_biped.star`), results are reliable but low-detail.

The fundamental issue: there is no level of detail between "featureless cylinder per bone" and "manually tuned extrusion steps." The quality cliff is steep.

---

## 4. The Specific Gap

### 4.1 Problem Statement

The extrusion step system is a continuous optimization problem:

> Given a semantic shape description ("muscular upper arm"), produce a sequence of `{extrude, scale, translate, rotate, tilt, bulge}` tuples that generates a mesh matching that description.

This is equivalent to asking an LLM to write a mathematical function that produces a specific 3D curve — a task for which LLMs have no reliable heuristics.

### 4.2 Parameter Count Analysis

For a single bone mesh with 3 extrusion steps, the parameter space is:

| Parameter | Per Step | Total (3 steps) |
|-----------|----------|------------------|
| `extrude` (fraction) | 1 | 3 (must sum to ~1.0) |
| `scale` (may be `[x,y]`) | 1-2 | 3-6 |
| `translate` | 0-3 | 0-9 |
| `rotate` | 0-1 | 0-3 |
| `tilt` | 0-2 | 0-6 |
| `bulge` | 0-2 | 0-6 |
| | | **3-33 per bone** |

A humanoid with 20 bones, averaging 3 steps each: **60-660 interdependent continuous parameters**.

Compare to the primitive-composition approach: each bone gets position + radius = **4 parameters per bone**, totaling **80 independent parameters** for 20 bones. These are independent — changing one doesn't affect others.

### 4.3 What's Needed

The gap can be expressed as a missing abstraction layer:

```
Current:    Intent → [manual tuning] → extrusion_steps → mesh
Needed:     Intent → shape_template(semantic_params) → extrusion_steps → mesh
```

The shape template layer would:
1. Accept semantic parameters LLMs can reason about (shape name, proportions, muscle definition level)
2. Compile deterministically to extrusion steps
3. Produce results that are always reasonable (no catastrophic deformations)
4. Allow expert override when needed (drop down to raw extrusion steps)

---

## 5. Comparison with External Systems

### 5.1 Three.js / Primitive Composition

**Strengths:** Direct mapping, independent parameters, graceful degradation.
**Weaknesses:** No skeletal animation, no skin weights, no deformation, triangle-soup topology, not deterministic.

SpecCade's advantage is that it produces **rigged, animatable meshes** with proper skin weights and deformation. Three.js scenes are static or require manual rigging. The goal is not to replicate Three.js but to bring its authoring ergonomics to SpecCade's superior output format.

### 5.2 Text-to-3D Models (DreamFusion, Zero-1-to-3, etc.)

**Strengths:** Natural language input, no parameter tuning.
**Weaknesses:** Non-deterministic, slow (minutes per asset), no skeletal structure, no animation support, quality varies wildly, not reproducible.

These are fundamentally incompatible with SpecCade's determinism guarantee (Hard Rule 1).

### 5.3 Procedural Generation Systems (Houdini, Geometry Nodes)

**Strengths:** Parameterized shape graphs, visual feedback, expert control.
**Weaknesses:** Require visual GUI for parameter tuning — same problem as extrusion steps when driven by text.

However, Houdini's approach of **named, pre-built shape operators** (tube, skin, loft, sweep) with semantic parameters is directly relevant. A "limb" operator in Houdini takes `length`, `taper`, `muscle_bulge` — exactly the abstraction level we need.

---

## 6. Recommendations

1. **Introduce a shape template system** that maps semantic parameters to extrusion step sequences. See companion proposal: `shape-templates-proposal.md`.

2. **Keep extrusion steps as the compilation target** — they are the right low-level primitive. The issue is only with direct authoring.

3. **Leverage attachments more aggressively** — the primitive attachment system already provides LLM-friendly composition. Shape templates should compose extrusion-based body shapes with attachment-based detail features.

4. **Design for LLM error tolerance** — shape templates should clamp parameters to valid ranges and guarantee reasonable output for any valid input combination.

5. **Provide worked examples** — LLMs learn from examples more than documentation. A library of shape template usage patterns (muscular, slender, stocky, etc.) would be more valuable than parameter reference docs.

---

## Appendix A: Metrics from SpecCade Specs

| Spec | Lines | Bones | Extrusion Steps (total) | Attachments | Perceived Quality |
|------|-------|-------|------------------------|-------------|-------------------|
| `simple_biped.star` | 62 | 18 | 0 | 0 | Low (featureless) |
| `preset_humanoid.star` | ~53 | 20 (preset) | 0 | 0 | Low (featureless) |
| `humanoid_male_v2.star` | 360 | 20 | ~45 | ~18 | Medium-High |

The complexity jump from "featureless" to "anatomically plausible" is roughly **6x in lines** and requires **45+ accumulated extrusion parameters** plus **18 attachment primitives** with precise 3D offsets.

## Appendix B: LLM Authoring Success Rates (Estimated)

Based on observed behavior across multiple sessions:

| Task | Estimated Success Rate |
|------|----------------------|
| Write a valid skeleton layout | 85-90% |
| Set reasonable uniform profile radii | 80-85% |
| Write correct mirror references | 95%+ |
| Write 2-3 step extrusion sequence with good proportions | 30-40% |
| Write 4+ step extrusion with anatomical detail | 10-15% |
| Place attachment primitives correctly on non-vertical bones | 20-30% |
| Produce a complete spec matching a specific character description | 5-10% |

The bottleneck is clear: extrusion step authoring and 3D offset placement account for the majority of failures.
