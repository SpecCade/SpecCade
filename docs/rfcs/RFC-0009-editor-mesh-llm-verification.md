# RFC-0009: Editor, Mesh Generation, and LLM Verification

**Status**: Draft
**Author**: Claude (Anthropic)
**Date**: 2026-01-17
**Target**: SpecCade Starlark Branch
**Depends On**: RFC-0008 (Audio Analysis Backend for preview feedback)

---

## Abstract

This RFC addresses interconnected gaps in SpecCade's authoring experience:

1. **Music DSL**: SpecCade has a sophisticated Pattern IR (Compose layer) with Euclidean rhythms, stacking, probability, and more—but these operators aren't yet exposed to Starlark. The gap is **Starlark bindings**, not missing functionality.
2. **Mesh Generation**: Limited to Blender primitives with no procedural (SDF/L-system) or character mesh support. Critically, there's no **LLM feedback loop** for mesh quality verification.
3. **Editor Tooling**: No interactive authoring environment for humans or LLM-in-the-loop workflows.
4. **LLM Mesh Verification**: Unlike audio (where spectral analysis provides feedback), meshes lack structured quality signals. This RFC proposes multi-view rendering + VLM evaluation, geometric metrics, and constraint-based validation.

The proposed solution is a unified editor application with real-time preview, Starlark Compose bindings, procedural mesh primitives, and multi-modal feedback for LLM agents.

---

## Part 1: Music DSL Gap Analysis

### Current State: Pattern IR (Compose Layer)

SpecCade has a sophisticated composition layer via `music.tracker_song_compose_v1`:

```json
{
  "op": "stack",
  "merge": "error",
  "parts": [
    {
      "op": "emit",
      "at": { "op": "euclid", "pulses": 4, "steps": 16, "stride": 4 },
      "cell": { "channel": 0, "note": "C4", "inst": 0, "vol": 64 }
    },
    {
      "op": "emit",
      "at": { "op": "pattern", "pattern": "....x.......x...", "stride": 1 },
      "cell": { "channel": 1, "note": "D2", "inst": 1, "vol": 64 }
    }
  ]
}
```

**Existing Capabilities**:

| Concept | Strudel | SpecCade Pattern IR |
|---------|---------|---------------------|
| Pattern mini-notation | `"bd sd hh hh"` | `{"op": "pattern", "pattern": "x...x..."}` |
| Euclidean rhythms | `"bd(3,8)"` | `{"op": "euclid", "pulses": 3, "steps": 8}` |
| Stacking | `.stack()`, `[a, b]` | `{"op": "stack", "parts": [...]}` |
| Probability | `"bd?0.5"` | `{"op": "prob", "p_permille": 500}` |
| Transforms | `.fast(2)` | `{"op": "repeat"}`, `{"op": "shift"}` |
| Sequence cycling | N/A | `{"op": "emit_seq", "note_seq": {...}}` |
| Named channels | N/A | `channel_ids` |
| Harmonic authoring | N/A | `harmony` + `pitch_seq` |
| Weighted choice | N/A | `{"op": "choose", "choices": [...]}` |
| Reusable defs | N/A | `{"op": "ref", "name": "four_on_floor"}` |

### Gap: Starlark Bindings for Compose IR

The Starlark `music/` module currently wraps only the low-level tracker API:

```python
# Current Starlark (low-level tracker API only)
tracker_pattern(64, notes = {"0": [pattern_note(0, "C4", 0), ...]})

# Missing: Starlark bindings for Compose operators
emit(at=euclid(4, 16), cell={...})  # NOT YET IN STARLARK
stack([kick_layer, snare_layer])     # NOT YET IN STARLARK
```

### Remaining Gaps vs Strudel

| Concept | Strudel | SpecCade Status |
|---------|---------|-----------------|
| Starlark bindings | N/A | Pattern IR exists but not exposed to Starlark |
| Live-coding REPL | Real-time eval | Not a goal (deterministic output) |
| Continuous time | Fractional beats | Row-based (discrete) |
| Pattern algebra | `+`, `*`, `|` operators | JSON operators only |
| Rich mini-notation | `"bd sd [hh hh]"` | Limited to `x.` characters |

### Solution: Starlark Compose Bindings

Expose Pattern IR operators to Starlark:

```python
# Proposed Starlark syntax
drums = stack([
    emit(at=euclid(4, 16), cell=cell(ch="kick", note="C4")),
    emit(at=pattern("....x.......x..."), cell=cell(ch="snare", note="D2")),
    emit(at=range_op(0, 2, 16), cell=cell(ch="hat", note="F#1"))
])

compose_song(
    bpm = 150,
    instruments = [kick_inst, snare_inst, hat_inst],
    patterns = {"verse": drums},
    arrangement = [arr_entry("verse", repeat=4)]
)
```

**Implementation**: ~800 LOC to wrap existing Compose IR in Starlark functions

---

## Part 2: Mesh Generation Gap Analysis

### Current State

Mesh generation is:
- **Blender-dependent** (Tier 2, non-deterministic)
- **Primitive-based**: cube, sphere, cylinder, cone, torus, plane, ico_sphere
- **Modifier stack**: bevel, subdivision, decimate, mirror, array, solidify

```python
# Current: Can only create modified primitives
mesh_recipe(
    "cube",
    [1.0, 1.0, 1.0],
    [bevel_modifier(0.05), subdivision_modifier(2)]
)
```

### What's Missing

| Category | Missing Capability |
|----------|-------------------|
| **Procedural** | L-systems, fractals, noise-based deformation |
| **Character** | Humanoid base mesh, rigging, blend shapes |
| **Organic** | Plant generation, terrain, rocks |
| **Architectural** | Modular building blocks, CSG operations |
| **Game-Ready** | LOD generation, UV unwrapping, texture baking |

### Research: Procedural Mesh Generation Approaches

1. **SDF-Based (Signed Distance Functions)**
   - Used by: ShaderToy, Inigo Quilez's work, Dreams (MediaMolecule)
   - Pro: Highly declarative, composable
   - Con: Requires meshing step (marching cubes)

2. **L-Systems**
   - Used by: SpeedTree, Houdini
   - Pro: Excellent for plants, fractals, organic structures
   - Con: Limited for mechanical/character meshes

3. **Parametric Surfaces**
   - Used by: CAD tools, OpenSCAD
   - Pro: Precise control, boolean operations
   - Con: Complex for organic shapes

4. **Neural Mesh Generation**
   - Used by: Point-E (OpenAI), Shap-E, DreamFusion
   - Pro: Text-to-3D possible
   - Con: Non-deterministic, heavy inference

**Recommendation**: SDF + L-Systems hybrid for Tier 1 (Rust-native, deterministic), keep Blender for Tier 2 complex cases.

---

## Part 3: LLM-in-the-Loop Mesh Generation

### The Core Problem

Unlike audio (where analysis metrics like spectral centroid provide feedback), mesh generation lacks clear "quality signals" that LLMs can use for iterative refinement. An LLM cannot perceive if a mesh "looks right."

### Approach 1: Multi-View Image Rendering + Vision-Language Models

Render the mesh from multiple viewpoints, then use a VLM (GPT-4o, Claude Vision) to evaluate:

```
┌─────────────────────────────────────────────────────────────┐
│  LLM Mesh Refinement Loop                                   │
│                                                             │
│  1. LLM generates mesh spec                                 │
│  2. SpecCade renders mesh → GLB                             │
│  3. Render GLB → 4 orthographic views (PNG)                 │
│  4. VLM evaluates: "Does this look like a sword?"           │
│  5. VLM provides structured feedback:                       │
│     - topology_score: 0.7                                   │
│     - silhouette_match: 0.85                                │
│     - issues: ["blade too thick", "handle too short"]       │
│  6. LLM refines spec based on feedback                      │
│  7. Repeat until quality threshold met                      │
└─────────────────────────────────────────────────────────────┘
```

**Implementation**:
```bash
# Proposed CLI
speccade render-views --spec sword.star --views 4 --output views/
speccade analyze-mesh --views views/ --prompt "medieval longsword"
```

**Output**:
```json
{
  "overall_match": 0.78,
  "silhouette_quality": 0.85,
  "proportions": {"blade_length": "good", "handle": "too_short"},
  "topology_issues": ["ngons on crossguard"],
  "suggestions": ["extend handle by 20%", "add fuller groove on blade"]
}
```

| Pros | Cons |
|------|------|
| VLMs are powerful visual reasoners | Requires VLM API calls (cost, latency) |
| Natural language feedback | 2D views may miss 3D issues |
| Works with any mesh format | VLM may hallucinate issues |
| Human-interpretable output | Needs well-designed prompts |

### Approach 2: Mesh Analysis Metrics (Deterministic)

Extract geometric properties without neural models:

```json
{
  "topology": {
    "vertices": 1247,
    "faces": 2490,
    "edges": 3735,
    "manifold": true,
    "watertight": true,
    "euler_characteristic": 2,
    "ngons": 0,
    "triangles": 2490,
    "quads": 0
  },
  "geometry": {
    "bounding_box": [1.2, 0.15, 0.08],
    "surface_area": 0.45,
    "volume": 0.012,
    "center_of_mass": [0.6, 0.0, 0.0],
    "aspect_ratio": 15.0
  },
  "quality": {
    "min_face_area": 0.0001,
    "max_face_area": 0.005,
    "face_area_variance": 0.0002,
    "edge_length_variance": 0.15,
    "smoothness_score": 0.92
  },
  "symmetry": {
    "x_symmetric": true,
    "y_symmetric": false,
    "z_symmetric": true
  }
}
```

LLM can reason: "Aspect ratio 15:1 matches sword expectations. Bounding box suggests blade is 1.2m long."

| Pros | Cons |
|------|------|
| Deterministic, fast | No semantic understanding |
| No external dependencies | Cannot judge "looks like a sword" |
| Good for topology checks | Requires domain knowledge to interpret |
| Useful for game-ready validation | Limited for creative evaluation |

### Approach 3: Reference Mesh Comparison

Compare generated mesh against reference examples:

```bash
speccade compare-mesh --generated sword.glb --reference examples/sword_ref.glb
```

**Output**:
```json
{
  "hausdorff_distance": 0.05,
  "chamfer_distance": 0.02,
  "iou_3d": 0.78,
  "silhouette_iou": {"front": 0.85, "side": 0.72, "top": 0.90},
  "topology_diff": {
    "vertex_count_ratio": 1.2,
    "face_distribution": "similar"
  }
}
```

| Pros | Cons |
|------|------|
| Objective comparison | Requires reference meshes |
| Works for style matching | May penalize valid variations |
| Standard metrics (Hausdorff, Chamfer) | Doesn't understand intent |

### Approach 4: Structured Constraints (Assertion-Based)

Define constraints in the spec that can be validated:

```python
sword = mesh_recipe(
    ...,
    constraints = [
        aspect_ratio_constraint(min=10, max=20),
        symmetry_constraint(axis="x"),
        bounding_box_constraint(x=[1.0, 1.5], y=[0.1, 0.2], z=[0.05, 0.1]),
        manifold_constraint(),
        max_faces_constraint(5000)
    ]
)
```

Validation produces:
```json
{
  "constraints": [
    {"name": "aspect_ratio", "passed": true, "value": 15.0},
    {"name": "symmetry_x", "passed": true},
    {"name": "bounding_box", "passed": false, "issue": "z dimension 0.12 exceeds max 0.1"},
    {"name": "manifold", "passed": true},
    {"name": "max_faces", "passed": true, "value": 2490}
  ],
  "all_passed": false
}
```

| Pros | Cons |
|------|------|
| Precise, verifiable | Constraints must be pre-defined |
| No ambiguity | Cannot express "looks good" |
| Fast, deterministic | May over-constrain creativity |
| LLM can iterate on failed constraints | Requires constraint authoring |

### Recommended: Hybrid Multi-Signal Approach

Combine all approaches for robust LLM mesh authoring:

```
┌─────────────────────────────────────────────────────────────────┐
│  Mesh Generation Pipeline with LLM-in-the-Loop                 │
│                                                                 │
│  ┌──────────────┐     ┌──────────────┐     ┌──────────────┐    │
│  │ 1. Generate  │ ──► │ 2. Validate  │ ──► │ 3. Render    │    │
│  │    Mesh Spec │     │    Constraints│     │    Views     │    │
│  └──────────────┘     └──────────────┘     └──────────────┘    │
│         │                    │                    │             │
│         │                    ▼                    ▼             │
│         │             ┌──────────────┐     ┌──────────────┐    │
│         │             │ Constraint   │     │ VLM Visual   │    │
│         │             │ Feedback     │     │ Evaluation   │    │
│         │             └──────────────┘     └──────────────┘    │
│         │                    │                    │             │
│         │                    └────────┬───────────┘             │
│         │                             │                         │
│         │                    ┌────────▼────────┐                │
│         │                    │ Combined Score  │                │
│         │                    │ + Suggestions   │                │
│         │                    └────────┬────────┘                │
│         │                             │                         │
│         │              ┌──────────────┴──────────────┐          │
│         │              │ Score > threshold?          │          │
│         │              └──────────────┬──────────────┘          │
│         │                    YES │           │ NO               │
│         │                        ▼           ▼                  │
│         │              ┌──────────────┐ ┌──────────────┐        │
│         └──────────────│   DONE ✓     │ │ LLM Refines  │────────┘
│                        └──────────────┘ │ Based on     │
│                                         │ Feedback     │
│                                         └──────────────┘
└─────────────────────────────────────────────────────────────────┘
```

---

## Part 4: Existing "Modeling with Code" Projects

### Comparison of Declarative 3D Authoring Tools

| Project | Language | Approach | Deterministic | LLM-Friendly |
|---------|----------|----------|---------------|--------------|
| **OpenSCAD** | Custom DSL | CSG + primitives | ✓ Yes | Medium (well-documented) |
| **CadQuery** | Python | B-Rep + sketches | ✓ Yes | Good (Python syntax) |
| **SolidPython** | Python | OpenSCAD wrapper | ✓ Yes | Good |
| **Blender + Python** | Python | Full 3D suite | ✗ No (floats) | Medium |
| **Three.js** | JavaScript | WebGL primitives | ✗ No | Good |
| **Curv** | Custom DSL | SDFs + F-Rep | ✓ Yes | Low (niche syntax) |
| **libfive** | Scheme/C++ | SDFs + CSG | ✓ Yes | Low (Scheme) |
| **Marching.js** | JavaScript | SDFs in WebGL | ✗ No | Medium |
| **Shader-based (ShaderToy)** | GLSL | SDF raymarching | ✓ Yes* | Low (GLSL) |

*Deterministic per GPU, may vary across hardware.

### Key Learnings for SpecCade

**OpenSCAD** (most popular declarative 3D):
- Simple primitive + boolean approach works for mechanical parts
- Weak for organic shapes
- Large community, good documentation
- LLMs can generate valid OpenSCAD (appears in training data)

```openscad
// OpenSCAD sword example
difference() {
    cube([100, 10, 5], center=true);  // blade
    translate([0, 0, 2]) cube([80, 6, 2], center=true);  // fuller
}
translate([-55, 0, 0]) cylinder(h=20, r=5);  // handle
```

**CadQuery** (Python-based CAD):
- Sketch-based workflow familiar to CAD users
- Powerful for precision parts
- Python syntax = LLM friendly
- Heavier dependency (OCC kernel)

```python
# CadQuery sword example
import cadquery as cq
blade = cq.Workplane("XY").box(100, 10, 5)
handle = cq.Workplane("XY").cylinder(20, 5).translate((-55, 0, 0))
sword = blade.union(handle)
```

**Curv / libfive** (SDF-focused):
- Mathematically elegant
- Great for organic shapes
- Less intuitive for beginners
- Small community

### Character Mesh Generation: State of the Art

For character/humanoid meshes, the landscape is different:

| Approach | Example | Quality | LLM-Usable |
|----------|---------|---------|------------|
| **Parametric rigs** | MakeHuman, MB-Lab | High | Possible (param sliders) |
| **Text-to-3D neural** | Shap-E, Point-E | Medium | Yes (text prompts) |
| **Image-to-3D** | TripoSR, LRM | Medium-High | Via VLM |
| **Mesh generation models** | MeshGPT, GET3D | Medium | Research only |
| **Template deformation** | SMPL, FLAME | High | Yes (params) |

**Challenge**: No existing tool provides deterministic, high-quality character meshes from pure code.

### SpecCade's Positioning

SpecCade should **not** try to compete with neural text-to-3D or professional modeling tools. Instead:

1. **Tier 1 (Deterministic)**: SDF-based procedural shapes for props, environment pieces, simple objects
2. **Tier 2 (Blender)**: Complex models via Geometry Nodes or scripting
3. **Tier 3 (External)**: Character meshes via parametric tools (MakeHuman) with spec-based customization

```python
# Proposed Tier 3: External character mesh integration
character = external_character(
    source = "makehuman",
    base = "adult_male_average",
    modifications = {
        "height": 1.85,
        "weight": 75,
        "muscle_tone": 0.6,
        "age": 30
    },
    style = "stylized_lowpoly",
    lod = 2
)
```

---

## Part 5: Editor Architecture

### Requirements

| Requirement | Human User | LLM Agent |
|-------------|------------|-----------|
| Text editing | Full editor | API/file access |
| Real-time preview | Visual feedback | Analysis metrics (RFC-001) |
| Error display | Inline diagnostics | Structured error JSON |
| Asset browser | Visual grid | Semantic search (RFC-001) |
| History | Undo/redo | Spec versioning |
| Iteration speed | <100ms preview | <500ms full + analyze |

### Architecture Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                     SpecCade Editor                              │
├─────────────────────────────┬───────────────────────────────────┤
│       Text Editor           │         Preview Panel             │
│  ┌──────────────────────┐   │   ┌───────────────────────────┐   │
│  │ Monaco/CodeMirror    │   │   │ Audio: Waveform + Play    │   │
│  │ - Starlark syntax    │   │   │ Texture: Image preview    │   │
│  │ - Autocomplete       │   │   │ Mesh: 3D viewport         │   │
│  │ - Inline errors      │   │   │ Music: Piano roll + Play  │   │
│  └──────────────────────┘   │   └───────────────────────────┘   │
├─────────────────────────────┼───────────────────────────────────┤
│      Asset Browser          │        Analysis Panel             │
│  ┌──────────────────────┐   │   ┌───────────────────────────┐   │
│  │ Preset library grid  │   │   │ Audio metrics (RFC-001)   │   │
│  │ Search + filter      │   │   │ Mesh stats (tris, verts)  │   │
│  │ Drag to insert       │   │   │ Budget usage gauge        │   │
│  └──────────────────────┘   │   └───────────────────────────┘   │
├─────────────────────────────┴───────────────────────────────────┤
│                    LLM Integration Panel (Optional)             │
│  ┌────────────────────────────────────────────────────────────┐ │
│  │ "Make the kick punchier" → [Suggested edits] → [Apply]    │ │
│  └────────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
```

### Technology Options

| Component | Option A | Option B | Option C |
|-----------|----------|----------|----------|
| **Framework** | Tauri (Rust + Web) | Electron | Native (egui) |
| **Text Editor** | Monaco | CodeMirror 6 | Custom |
| **3D Viewport** | three.js | Bevy (WASM) | wgpu direct |
| **Audio** | Web Audio API | cpal (native) | Hybrid |

**Recommendation**: Tauri + Monaco + three.js + Web Audio

**Rationale**:
- Tauri: Rust backend matches SpecCade core, smaller binary than Electron
- Monaco: VS Code quality, excellent autocomplete infrastructure
- three.js: Mature, well-documented, handles glTF natively
- Web Audio: Sufficient for preview playback

### LLM Integration Modes

#### Mode 1: Chat-Driven Editing

```
User: "Add more sub-bass to the kick"
LLM: [Analyzes current spec]
     [Proposes diff: layers[0].filter.cutoff: 200 → 80]
     [Shows preview comparison]
User: [Accepts/Rejects]
```

This is human-in-the-loop; LLM suggests, human approves.

#### Mode 2: Autonomous Refinement

```
User: "Make this sound like a 909 kick" [provides reference.wav]
LLM: [Loop]:
       1. Generate candidate spec
       2. Render audio
       3. Analyze vs reference (RFC-001)
       4. If similarity > threshold: done
       5. Else: refine spec, goto 1
```

This requires RFC-001 audio analysis backend.

#### Mode 3: Hybrid (Recommended)

- LLM operates on analysis metrics, not raw audio
- Human sees visual diff of changes
- LLM can iterate autonomously up to N steps
- Human approves final result

---

## Proposed Solutions

### S1: Music Pattern Mini-Notation

#### Problem Addressed
Current tracker API is verbose (15 lines vs 1 line for simple patterns).

#### Option A: Strudel-Compatible Parser

Implement Strudel mini-notation parser in Rust:

```python
# New syntax
drums = pattern("bd sd [hh hh] sd")
bass = pattern("c2 ~ c2 [eb2 g2]").slow(2)
song = drums.stack(bass)
```

Parser converts mini-notation → tracker pattern IR.

| Pros | Cons |
|------|------|
| LLMs know Strudel syntax | Parser complexity |
| Concise, composable | May not cover all tracker features |
| Large existing documentation | Two syntaxes to maintain |

**Implementation**: ~1500 LOC parser + ~500 LOC pattern transforms

#### Option B: Rhythm DSL (Simpler)

Custom DSL focused on common game audio patterns:

```python
drums = beat("x...x...x...x...", kick, step=1/16)  # x = hit, . = rest
snare = beat("....x.......x...", snare, step=1/16)
hats = beat("x.x.x.x.x.x.x.x.", hihat, step=1/16)

song = layer(drums, snare, hats)
```

| Pros | Cons |
|------|------|
| Simple to implement | Less expressive than Strudel |
| Easy to learn | Novel syntax (less LLM training data) |
| Direct mapping to trackers | No polymetric support |

**Implementation**: ~400 LOC

#### Option C: Euclidean + Templates

Pre-built pattern templates with Euclidean rhythm support:

```python
drums = drum_pattern(
    style = "four_on_floor",
    kick = euclidean(4, 16),    # 4 hits over 16 steps
    snare = euclidean(2, 16, rotate=4),
    hihat = euclidean(8, 16)
)
```

| Pros | Cons |
|------|------|
| Very concise for common cases | Limited to templates |
| Euclidean rhythms are well-studied | Custom patterns need fallback |
| Matches RFC-001 semantic approach | May feel restrictive |

**Implementation**: ~600 LOC

**Recommendation**: Option A (Strudel-compatible) for full expressiveness, with Option C templates as high-level sugar.

---

### S2: Procedural Mesh Generation

#### Problem Addressed
Mesh generation limited to modified primitives, no procedural or character support.

#### Option A: SDF Mesh Library (Tier 1)

Implement SDF primitives and operations in pure Rust:

```python
# SDF-based mesh
body = sdf_sphere(1.0)
head = sdf_sphere(0.6).translate(0, 1.2, 0)
character = sdf_union(body, head, smoothness=0.1)

mesh_from_sdf(character, resolution=64)  # Marching cubes
```

Operations:
- Primitives: sphere, box, cylinder, torus, plane
- Combine: union, intersection, difference (smooth variants)
- Transform: translate, rotate, scale
- Modify: round, onion, elongate, twist, bend

| Pros | Cons |
|------|------|
| Tier 1 deterministic | Limited organic detail |
| Composable, declarative | Resolution/quality tradeoff |
| No external dependencies | Requires marching cubes |
| Fast iteration | UV mapping complex |

**Implementation**: ~2000 LOC SDF library + ~800 LOC marching cubes

#### Option B: L-System Module

Grammar-based generation for organic structures:

```python
# L-system tree
tree = lsystem(
    axiom = "F",
    rules = {
        "F": "FF+[+F-F-F]-[-F+F+F]"
    },
    iterations = 4,
    angle = 25,
    segment_length = 0.1
)

mesh_from_lsystem(tree, branch_radius=0.02)
```

| Pros | Cons |
|------|------|
| Excellent for plants/fractals | Complex rule authoring |
| Deterministic | Not suitable for characters |
| Compact representation | Limited control over details |

**Implementation**: ~1200 LOC

#### Option C: Modular Character Kit

Pre-built character parts with parametric customization:

```python
character = humanoid_base(
    height = 1.8,
    proportions = "athletic",
    style = "stylized"  # realistic, stylized, chibi
)

character = character.with_head(
    shape = "round",
    features = ["pointed_ears", "large_eyes"]
)

mesh_from_character(character, lod=2)
```

| Pros | Cons |
|------|------|
| Production-ready output | Large asset library needed |
| Easy for non-artists | Limited customization |
| Consistent style | Significant upfront work |

**Implementation**: ~1500 LOC code + extensive asset library

#### Option D: Enhanced Blender Pipeline (Tier 2)

Keep Blender but add:
- Geometry Nodes integration
- Python scripting for procedural generation
- Asset library with parametric models

| Pros | Cons |
|------|------|
| Full Blender power | Still Tier 2 (non-deterministic) |
| Existing ecosystem | Blender version dependency |
| Professional quality | Heavy runtime |

**Recommendation**:
- Option A (SDF) for Tier 1 procedural shapes
- Option B (L-Systems) for organic/plants
- Option D (Enhanced Blender) for character meshes until Option C assets exist

---

### S3: Editor Application

#### Problem Addressed
No interactive authoring environment for iteration and preview.

#### Option A: Tauri Desktop Application

Native app using Tauri (Rust backend, web frontend):

```
┌─ src-tauri/          # Rust backend
│  ├─ speccade-cli integration
│  ├─ file watching
│  └─ preview generation
│
└─ src/                # Web frontend
   ├─ Monaco editor
   ├─ three.js viewport
   └─ Web Audio preview
```

| Pros | Cons |
|------|------|
| Full SpecCade integration | Desktop-only |
| Native performance | Two codebases (Rust + JS) |
| Small binary (~15MB) | Web tech complexity |
| Cross-platform | Tauri still maturing |

**Implementation**: ~3000 LOC Rust + ~5000 LOC TypeScript

#### Option B: VS Code Extension

Extension providing:
- Starlark language support
- Preview panel
- Asset browser sidebar

| Pros | Cons |
|------|------|
| Familiar environment | Limited custom UI |
| Extension marketplace | VS Code dependency |
| Fast to develop | Preview panel limitations |
| Existing user base | |

**Implementation**: ~2000 LOC TypeScript

#### Option C: Web Application

Browser-based editor with WASM SpecCade:

```
┌─ Browser ──────────────────────────────────────────┐
│  Monaco Editor │ Preview (three.js + Web Audio)   │
│                │                                   │
│  ─────────────────────────────────────────────────│
│  speccade-wasm: compile + generate in browser     │
└───────────────────────────────────────────────────┘
```

| Pros | Cons |
|------|------|
| Zero install | WASM compilation complexity |
| Shareable links | Browser limitations |
| Works on tablets | Offline support harder |
| LLM cloud integration easy | Large WASM bundle |

**Implementation**: ~2000 LOC TypeScript + WASM build setup

#### Option D: Hybrid (Recommended)

Tauri app that can also run as web app:

- **Desktop mode**: Full native integration, file system access
- **Web mode**: Subset features, cloud storage

Shared frontend codebase, conditional backend.

**Implementation**: ~4000 LOC (shared frontend) + ~2000 LOC (Tauri) + ~500 LOC (web adapter)

**Recommendation**: Option D (Hybrid Tauri + Web)

---

### S4: LLM-Editor Integration Protocol

#### Problem Addressed
LLMs cannot operate the editor, but could provide assistance through a protocol.

#### Design: Language Server Protocol + Custom Extensions

```typescript
interface SpecCadeLSP extends LSP {
  // Standard LSP
  textDocument/completion
  textDocument/diagnostics

  // Custom extensions
  speccade/preview      // Request preview render
  speccade/analyze      // Get audio/mesh analysis
  speccade/suggest      // LLM-powered suggestions
  speccade/refine       // Autonomous refinement
}
```

#### LLM Integration Points

1. **Autocomplete Enhancement**
   - Standard LSP completion for syntax
   - LLM-powered completion for semantic suggestions
   - Example: typing `sound(character=` triggers LLM to suggest "warm", "bright", etc.

2. **Natural Language Commands**
   ```
   User: "/make the bass heavier"
   Editor: [Parses intent] → [Identifies bass layer] → [Suggests changes]
   ```

3. **Reference-Based Editing**
   ```
   User: [Drops reference.wav into editor]
   Editor: "Match this sound?"
   User: "Yes"
   Editor: [Runs RFC-001 analysis] → [LLM generates spec] → [Iterates to match]
   ```

4. **Diff Preview**
   - LLM changes shown as diff
   - Side-by-side audio comparison
   - One-click accept/reject

#### Non-Goals

The editor does NOT aim to:
- Have LLM directly manipulate GUI (unreliable)
- Replace human judgment for quality
- Auto-commit changes

---

## Implementation Roadmap

### Phase 1: Music DSL (4 weeks)

| Week | Deliverable |
|------|-------------|
| 1 | Mini-notation parser (subset) |
| 2 | Pattern transforms (fast, slow, rev) |
| 3 | Euclidean rhythms + templates |
| 4 | Integration tests + documentation |

### Phase 2: SDF Mesh (6 weeks)

| Week | Deliverable |
|------|-------------|
| 1-2 | SDF primitives + operations |
| 3 | Marching cubes mesher |
| 4 | Starlark bindings |
| 5 | L-system module |
| 6 | Integration + golden tests |

### Phase 3: Editor MVP (8 weeks)

| Week | Deliverable |
|------|-------------|
| 1-2 | Tauri scaffold + Monaco integration |
| 3 | Starlark syntax highlighting + diagnostics |
| 4 | Audio preview (waveform + playback) |
| 5 | Texture preview |
| 6 | 3D viewport (three.js + glTF) |
| 7 | Asset browser + preset library |
| 8 | Polish + packaging |

### Phase 4: LLM Integration (4 weeks)

| Week | Deliverable |
|------|-------------|
| 1 | LSP server skeleton |
| 2 | speccade/suggest endpoint |
| 3 | Natural language command parser |
| 4 | Autonomous refinement loop |

---

## Success Metrics

### Music DSL

| Metric | Target |
|--------|--------|
| Pattern LOC reduction | 10x (150 → 15 lines for typical song) |
| LLM pattern generation accuracy | >85% valid on first attempt |
| Mini-notation coverage | 80% of Strudel core syntax |

### Mesh Generation

| Metric | Target |
|--------|--------|
| SDF operation set | 15+ primitives, 10+ operations |
| Mesh quality | <5% deviation from reference at res=64 |
| Generation time | <500ms for 10k triangles |

### Editor

| Metric | Target |
|--------|--------|
| Preview latency | <100ms for audio, <200ms for mesh |
| First-run setup | <2 minutes |
| Crash rate | <1% of sessions |

### LLM Integration

| Metric | Target |
|--------|--------|
| Suggestion acceptance rate | >60% |
| Refinement iterations to match reference | <5 average |
| Natural language command success | >80% |

---

## Appendix A: Mini-Notation Grammar

Subset of Strudel grammar to implement:

```ebnf
pattern     = sequence | stack | polymetric
sequence    = element (" " element)*
stack       = "[" pattern ("," pattern)* "]"
polymetric  = "{" pattern ("," pattern)* "}"

element     = atom modifiers?
atom        = note | rest | group
note        = SOUND_NAME | PITCH
rest        = "~"
group       = "[" sequence "]"

modifiers   = modifier+
modifier    = "*" NUMBER      # repeat
            | "/" NUMBER      # slow
            | "?" NUMBER?     # probability
            | "@" NUMBER      # weight
            | "(" NUMBER "," NUMBER ("," NUMBER)? ")"  # euclidean

SOUND_NAME  = [a-z]+
PITCH       = [A-Ga-g][#b]?[0-9]
NUMBER      = [0-9]+("."[0-9]+)?
```

### Examples

| Mini-notation | Meaning |
|---------------|---------|
| `bd sd bd sd` | Four quarter notes |
| `bd*4` | Kick repeated 4 times |
| `[bd, sd]` | Kick and snare stacked |
| `bd(3,8)` | Euclidean: 3 hits over 8 steps |
| `bd?0.5` | 50% probability |
| `[bd sd] hh*2` | Group + repeat |

---

## Appendix B: SDF Operations Reference

### Primitives

| Function | Parameters | Description |
|----------|------------|-------------|
| `sdf_sphere` | radius | Centered sphere |
| `sdf_box` | x, y, z | Axis-aligned box |
| `sdf_cylinder` | radius, height | Y-aligned cylinder |
| `sdf_torus` | major_r, minor_r | XZ-plane torus |
| `sdf_cone` | angle, height | Y-aligned cone |
| `sdf_capsule` | a, b, radius | Line segment capsule |

### Combinations

| Function | Parameters | Description |
|----------|------------|-------------|
| `sdf_union` | a, b, k? | Combine shapes (smooth if k) |
| `sdf_subtract` | a, b, k? | a - b |
| `sdf_intersect` | a, b, k? | a ∩ b |

### Transforms

| Function | Parameters | Description |
|----------|------------|-------------|
| `.translate` | x, y, z | Move shape |
| `.rotate` | x, y, z | Euler rotation (degrees) |
| `.scale` | factor | Uniform scale |
| `.mirror` | axis | Mirror across axis |

### Deformations

| Function | Parameters | Description |
|----------|------------|-------------|
| `.round` | radius | Round edges |
| `.onion` | thickness | Hollow shell |
| `.twist` | amount | Twist around Y |
| `.bend` | amount | Bend around Y |

---

## Appendix C: Editor Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| Ctrl+Enter | Generate preview |
| Ctrl+Shift+Enter | Generate + analyze |
| Ctrl+Space | Autocomplete |
| Ctrl+P | Command palette |
| Ctrl+Shift+P | LLM command input |
| Ctrl+S | Save spec |
| Ctrl+E | Export asset |
| Space (in preview) | Play/pause audio |
| Mouse drag (in 3D) | Orbit camera |

---

## References

1. McLean, A. (2014). "Making Programming Languages to Dance to: Live Coding with Tidal." FARM.
2. Roberts, C., et al. (2019). "Bringing the Web to the Dance Floor: Developing Strudel." NIME.
3. Hart, J. C. (1996). "Sphere Tracing: A Geometric Method for Antialiased Ray Tracing." The Visual Computer.
4. Prusinkiewicz, P., & Lindenmayer, A. (1990). "The Algorithmic Beauty of Plants." Springer.
5. Chen, M., et al. (2023). "Teaching Large Language Models to Self-Debug." arXiv.
6. Lorensen, W. E., & Cline, H. E. (1987). "Marching Cubes: A High Resolution 3D Surface Construction Algorithm." SIGGRAPH.

---

*End of RFC-002*
