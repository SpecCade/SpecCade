# Structural Metrics for LLM-Friendly 3D Feedback

**Date:** 2026-01-27
**Status:** Ready for implementation
**Goal:** Add non-opinionated structural metrics to mesh generation reports so LLMs can self-correct 3D output.

## Problem

LLMs can generate audio/texture specs that produce quality results, but 3D assets (meshes, characters, animations) are poor. The core issue: LLMs can't verify whether generated geometry matches their intent.

Current reports include topology metrics (vertex_count, triangle_count, manifold) but not structural metrics (proportions, symmetry, balance, component relationships).

## Solution

Add structural metrics to `OutputMetrics` that describe geometric properties without encoding aesthetic opinions. LLMs compare these against their stated intent to self-correct.

## Metrics to Implement

### 1. Universal Geometry Metrics (all meshes)

```rust
pub struct GeometryMetrics {
    /// Bounding box extent [x, y, z] in meters
    pub extent: [f64; 3],
    /// Aspect ratios: xy, xz, yz
    pub aspect_ratios: AspectRatios,
    /// Which axis is longest: "X", "Y", or "Z"
    pub dominant_axis: String,
    /// Ratio of longest to shortest dimension
    pub elongation: f64,
    /// Centroid position [x, y, z]
    pub centroid: [f64; 3],
    /// Centroid as fraction of bounding box [0-1, 0-1, 0-1]
    pub centroid_normalized: [f64; 3],
    /// Ratio of mesh volume to convex hull volume (0-1)
    pub convex_hull_ratio: f64,
}

pub struct AspectRatios {
    pub xy: f64,
    pub xz: f64,
    pub yz: f64,
}
```

### 2. Symmetry Metrics (all meshes)

```rust
pub struct SymmetryMetrics {
    /// Symmetry score for X axis reflection (0-1)
    pub x_axis: f64,
    /// Symmetry score for Y axis reflection (0-1)
    pub y_axis: f64,
    /// Symmetry score for Z axis reflection (0-1)
    pub z_axis: f64,
}
```

### 3. Component Metrics (multi-part meshes)

```rust
pub struct ComponentMetrics {
    /// Per-component breakdown
    pub components: Vec<ComponentInfo>,
    /// Adjacency/overlap between components
    pub adjacency: Vec<ComponentAdjacency>,
}

pub struct ComponentInfo {
    pub name: String,
    pub bounds_min: [f64; 3],
    pub bounds_max: [f64; 3],
    pub volume_fraction: f64,
    pub centroid: [f64; 3],
    pub triangle_count: u32,
}

pub struct ComponentAdjacency {
    pub part_a: String,
    pub part_b: String,
    /// Gap between parts (positive = gap, negative = overlap)
    pub distance: f64,
}
```

### 4. Skeletal Metrics (rigged meshes)

```rust
pub struct SkeletalStructureMetrics {
    /// Bone hierarchy depth
    pub hierarchy_depth: u32,
    /// Terminal bone names (leaves)
    pub terminal_bones: Vec<String>,
    /// Per-bone mesh coverage
    pub bone_coverage: Vec<BoneCoverageInfo>,
    /// Left/right bone pair symmetry
    pub bone_symmetry: Vec<BonePairSymmetry>,
}

pub struct BoneCoverageInfo {
    pub bone_name: String,
    pub bone_length: f64,
    pub mesh_length_along_bone: f64,
    pub coverage_ratio: f64,
    pub mesh_radius_avg: f64,
}

pub struct BonePairSymmetry {
    pub bone_left: String,
    pub bone_right: String,
    pub length_ratio: f64,
    pub radius_ratio: f64,
}
```

### 5. Scale Reference (all meshes)

```rust
pub struct ScaleReference {
    /// Longest dimension in meters
    pub longest_dimension_m: f64,
    /// Total volume in cubic meters
    pub volume_m3: f64,
    /// Fits in a 1m cube?
    pub fits_in_1m_cube: bool,
    /// Fits in a 10cm cube? (handheld object scale)
    pub fits_in_10cm_cube: bool,
}
```

## Implementation Plan

### Phase 1: Rust Types (speccade-spec)

**File:** `crates/speccade-spec/src/report/structural.rs`

1. Add new module with all metric structs
2. Add `structural: Option<StructuralMetrics>` to `OutputMetrics`
3. Export from `report/mod.rs`

**Estimated scope:** ~200 lines of Rust types + builders

### Phase 2: Blender Computation (Python)

**File:** `blender/structural_metrics.py` (new)

1. `compute_geometry_metrics(obj)` - extent, aspect ratios, centroid, convex hull
2. `compute_symmetry_metrics(obj)` - axis reflection scores via vertex sampling
3. `compute_component_metrics(objects)` - per-object breakdown and adjacency
4. `compute_skeletal_metrics(armature, mesh)` - bone coverage and symmetry
5. `compute_scale_reference(obj)` - dimension checks

**File:** `blender/entrypoint.py`

1. Call structural metrics functions after mesh generation
2. Include results in JSON output

**Estimated scope:** ~400 lines of Python

### Phase 3: Backend Integration

**File:** `crates/speccade-backend-blender/src/lib.rs`

1. Parse structural metrics from Blender JSON output
2. Populate `OutputMetrics.structural` field

**Estimated scope:** ~100 lines of Rust parsing

### Phase 4: Documentation

**File:** `docs/spec-reference/structural-metrics.md`

1. Document each metric with examples
2. Show how LLMs should interpret metrics against intent

**Estimated scope:** ~150 lines of markdown

## File Touch Points

| File | Action |
|------|--------|
| `crates/speccade-spec/src/report/mod.rs` | Add structural module export |
| `crates/speccade-spec/src/report/structural.rs` | NEW: metric types |
| `crates/speccade-spec/src/report/output.rs` | Add structural field to OutputMetrics |
| `blender/structural_metrics.py` | NEW: computation functions |
| `blender/entrypoint.py` | Call structural metrics |
| `crates/speccade-backend-blender/src/lib.rs` | Parse structural metrics |
| `docs/spec-reference/structural-metrics.md` | NEW: documentation |

## Task Breakdown

1. **Task 1:** Add Rust types for structural metrics
2. **Task 2:** Implement Blender Python computation
3. **Task 3:** Wire Blender output through backend
4. **Task 4:** Add documentation
5. **Task 5:** Add basic tests

## Verification

After implementation:
```bash
# Generate a mesh and check report includes structural metrics
cargo run -p speccade-cli -- generate --spec golden/starlark/character_humanoid.star --out-root ./test-out

# Check report has structural section
cat test-out/characters/humanoid.report.json | jq '.outputs[0].metrics.structural'
```

## Out of Scope

- Constraints based on structural metrics (future work)
- Editor visualization of metrics (future work)
- Animation-specific motion metrics (separate task)
