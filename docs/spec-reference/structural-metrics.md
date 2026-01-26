# Structural Metrics Reference

Structural metrics provide factual geometric analysis of generated meshes. They enable LLMs to verify whether generated 3D assets match their stated intent without encoding aesthetic opinions.

| Property | Value |
|----------|-------|
| Applies To | `static_mesh`, `skeletal_mesh` |
| Report Field | `metrics.structural` |
| Purpose | LLM self-correction feedback |

## Overview

When generating 3D assets, LLMs cannot visually inspect the results. Structural metrics bridge this gap by providing quantitative descriptions of:

- **Shape** - Bounding box dimensions, aspect ratios, elongation
- **Balance** - Centroid position, convex hull ratio
- **Symmetry** - Reflection scores across each axis
- **Components** - Per-part breakdown and spatial relationships
- **Skeleton** - Bone hierarchy, coverage, and symmetry (rigged meshes only)
- **Scale** - Real-world size classification

These metrics are facts, not judgments. An LLM compares them against its intent to determine whether the output matches expectations.

## Geometry Metrics

Universal measurements applicable to all meshes.

### Fields

| Field | Type | Description |
|-------|------|-------------|
| `extent` | `[f64; 3]` | Bounding box dimensions [X, Y, Z] in meters |
| `aspect_ratios.xy` | f64 | Ratio of X extent to Y extent |
| `aspect_ratios.xz` | f64 | Ratio of X extent to Z extent |
| `aspect_ratios.yz` | f64 | Ratio of Y extent to Z extent |
| `dominant_axis` | string | Longest axis: `"X"`, `"Y"`, or `"Z"` |
| `elongation` | f64 | Ratio of longest to shortest dimension (>= 1.0) |
| `centroid` | `[f64; 3]` | Center of mass [X, Y, Z] in world coordinates |
| `centroid_normalized` | `[f64; 3]` | Centroid as fraction of bounding box [0-1, 0-1, 0-1] |
| `convex_hull_ratio` | f64 | Mesh volume / convex hull volume (0-1) |

### JSON Example

```json
{
  "geometry": {
    "extent": [0.4, 0.3, 1.8],
    "aspect_ratios": {
      "xy": 1.33,
      "xz": 0.22,
      "yz": 0.17
    },
    "dominant_axis": "Z",
    "elongation": 6.0,
    "centroid": [0.0, 0.0, 0.9],
    "centroid_normalized": [0.5, 0.5, 0.5],
    "convex_hull_ratio": 0.65
  }
}
```

### Interpretation Guide

| Metric | How to Use |
|--------|------------|
| `extent` | Verify dimensions match intent (e.g., "1.8m tall humanoid" should have Z near 1.8) |
| `aspect_ratios` | Check proportions (e.g., a "tall thin character" should have low xy and xz ratios) |
| `dominant_axis` | Confirm orientation (e.g., a standing character should have dominant_axis = "Z") |
| `elongation` | Distinguish shapes (sphere ~1.0, cylinder ~2-3, sword ~10+) |
| `centroid_normalized` | Detect imbalance (e.g., [0.5, 0.5, 0.3] means mass is concentrated in lower third) |
| `convex_hull_ratio` | Assess complexity (simple shapes ~0.9+, characters with limbs ~0.4-0.7) |

## Symmetry Metrics

Reflection symmetry scores for each axis.

### Fields

| Field | Type | Description |
|-------|------|-------------|
| `x_axis` | f64 | Symmetry score for reflection across YZ plane (0-1) |
| `y_axis` | f64 | Symmetry score for reflection across XZ plane (0-1) |
| `z_axis` | f64 | Symmetry score for reflection across XY plane (0-1) |

A score of 1.0 indicates perfect symmetry; 0.0 indicates no symmetry.

### JSON Example

```json
{
  "symmetry": {
    "x_axis": 0.95,
    "y_axis": 0.12,
    "z_axis": 0.08
  }
}
```

### Interpretation Guide

| Intent | Expected Scores |
|--------|-----------------|
| Bilateral symmetry (humanoid) | `x_axis` > 0.9, others low |
| Radial symmetry (pillar) | `x_axis` and `y_axis` both high |
| Asymmetric (organic rock) | All scores low (<0.5) |
| Perfect cube | All scores > 0.95 |

If intent was "symmetric humanoid" but `x_axis` < 0.8, the mesh may have asymmetric limbs or features that need correction.

## Component Metrics

Per-part breakdown for multi-component meshes.

### Fields

**ComponentInfo:**

| Field | Type | Description |
|-------|------|-------------|
| `name` | string | Component/object name |
| `bounds_min` | `[f64; 3]` | Bounding box minimum corner [X, Y, Z] |
| `bounds_max` | `[f64; 3]` | Bounding box maximum corner [X, Y, Z] |
| `volume_fraction` | f64 | Fraction of total mesh volume (0-1) |
| `centroid` | `[f64; 3]` | Component center of mass [X, Y, Z] |
| `triangle_count` | u32 | Number of triangles in this component |

**ComponentAdjacency:**

| Field | Type | Description |
|-------|------|-------------|
| `part_a` | string | First component name |
| `part_b` | string | Second component name |
| `distance` | f64 | Gap between components in meters (negative = overlap) |

### JSON Example

```json
{
  "components": {
    "components": [
      {
        "name": "body",
        "bounds_min": [-0.2, -0.15, 0.0],
        "bounds_max": [0.2, 0.15, 1.2],
        "volume_fraction": 0.6,
        "centroid": [0.0, 0.0, 0.6],
        "triangle_count": 2400
      },
      {
        "name": "head",
        "bounds_min": [-0.12, -0.12, 1.2],
        "bounds_max": [0.12, 0.12, 1.5],
        "volume_fraction": 0.15,
        "centroid": [0.0, 0.0, 1.35],
        "triangle_count": 800
      }
    ],
    "adjacency": [
      {
        "part_a": "body",
        "part_b": "head",
        "distance": 0.0
      }
    ]
  }
}
```

### Interpretation Guide

| Check | What to Look For |
|-------|------------------|
| Component count | Matches expected parts (e.g., humanoid should have body, head, arms, legs) |
| Volume fractions | Proportional to intent (e.g., head shouldn't be 50% of total volume) |
| Adjacency distances | Positive = gap (may indicate disconnected parts), negative = overlap (expected for connected parts), zero = touching |
| Relative positions | Components in expected spatial arrangement (head above body, arms at sides) |

## Skeletal Structure Metrics

Metrics for rigged meshes with armatures. Only present when the mesh has a skeleton.

### Fields

**SkeletalStructureMetrics:**

| Field | Type | Description |
|-------|------|-------------|
| `hierarchy_depth` | u32 | Maximum depth of bone hierarchy (root = depth 1) |
| `terminal_bones` | array | Names of leaf bones (no children) |
| `bone_coverage` | array | Per-bone mesh coverage info |
| `bone_symmetry` | array | Left/right bone pair comparisons |

**BoneCoverageInfo:**

| Field | Type | Description |
|-------|------|-------------|
| `bone_name` | string | Name of the bone |
| `bone_length` | f64 | Length of the bone in meters |
| `mesh_length_along_bone` | f64 | Length of mesh geometry along bone axis in meters |
| `coverage_ratio` | f64 | Ratio of mesh length to bone length |
| `mesh_radius_avg` | f64 | Average radius of mesh geometry around bone in meters |

**BonePairSymmetry:**

| Field | Type | Description |
|-------|------|-------------|
| `bone_left` | string | Left bone name (e.g., "arm.L") |
| `bone_right` | string | Right bone name (e.g., "arm.R") |
| `length_ratio` | f64 | Left bone length / right bone length |
| `radius_ratio` | f64 | Left mesh radius / right mesh radius |

### JSON Example

```json
{
  "skeletal": {
    "hierarchy_depth": 5,
    "terminal_bones": [
      "hand.L", "hand.R",
      "foot.L", "foot.R",
      "head"
    ],
    "bone_coverage": [
      {
        "bone_name": "spine",
        "bone_length": 0.4,
        "mesh_length_along_bone": 0.45,
        "coverage_ratio": 1.125,
        "mesh_radius_avg": 0.15
      },
      {
        "bone_name": "upper_arm.L",
        "bone_length": 0.28,
        "mesh_length_along_bone": 0.26,
        "coverage_ratio": 0.93,
        "mesh_radius_avg": 0.05
      }
    ],
    "bone_symmetry": [
      {
        "bone_left": "upper_arm.L",
        "bone_right": "upper_arm.R",
        "length_ratio": 1.0,
        "radius_ratio": 1.02
      },
      {
        "bone_left": "thigh.L",
        "bone_right": "thigh.R",
        "length_ratio": 1.0,
        "radius_ratio": 0.98
      }
    ]
  }
}
```

### Interpretation Guide

| Check | What to Look For |
|-------|------------------|
| `hierarchy_depth` | Humanoid typically 4-6, simple rig 2-3 |
| `terminal_bones` | Expected endpoints present (hands, feet, head for humanoid) |
| `coverage_ratio` | Near 1.0 means mesh matches bone; <0.5 may indicate missing geometry |
| `mesh_radius_avg` | Proportional to body part (arm ~0.05m, torso ~0.15m) |
| `length_ratio` / `radius_ratio` | Near 1.0 for symmetric characters; deviations indicate asymmetry |

If intent was "symmetric humanoid" but `radius_ratio` for arms is 0.7, one arm is significantly thinner than the other.

## Scale Reference

Quick size classification for real-world scale verification.

### Fields

| Field | Type | Description |
|-------|------|-------------|
| `longest_dimension_m` | f64 | Longest bounding box dimension in meters |
| `volume_m3` | f64 | Total bounding box volume in cubic meters |
| `fits_in_1m_cube` | bool | Whether mesh fits within a 1 meter cube |
| `fits_in_10cm_cube` | bool | Whether mesh fits within a 10 centimeter cube |

### JSON Example

```json
{
  "scale": {
    "longest_dimension_m": 1.8,
    "volume_m3": 0.216,
    "fits_in_1m_cube": false,
    "fits_in_10cm_cube": false
  }
}
```

### Interpretation Guide

| `fits_in_*` Pattern | Typical Objects |
|---------------------|-----------------|
| Both true | Small props: coins, keys, small tools |
| 1m true, 10cm false | Medium props: weapons, handheld items, small furniture |
| Both false | Large objects: characters, furniture, vehicles |

| Longest Dimension | Typical Scale |
|-------------------|---------------|
| < 0.1m | Small prop (handheld) |
| 0.1m - 1.0m | Medium prop (furniture, weapons) |
| 1.0m - 2.0m | Character-scale (humanoid, large furniture) |
| > 2.0m | Large-scale (vehicles, buildings) |

## Example Report

A complete structural metrics section for a humanoid character:

```json
{
  "metrics": {
    "vertex_count": 5200,
    "triangle_count": 8400,
    "manifold": true,
    "structural": {
      "geometry": {
        "extent": [0.45, 0.28, 1.78],
        "aspect_ratios": {
          "xy": 1.61,
          "xz": 0.25,
          "yz": 0.16
        },
        "dominant_axis": "Z",
        "elongation": 6.36,
        "centroid": [0.0, 0.02, 0.85],
        "centroid_normalized": [0.5, 0.54, 0.48],
        "convex_hull_ratio": 0.42
      },
      "symmetry": {
        "x_axis": 0.94,
        "y_axis": 0.15,
        "z_axis": 0.11
      },
      "components": {
        "components": [
          {
            "name": "body",
            "bounds_min": [-0.18, -0.12, 0.0],
            "bounds_max": [0.18, 0.12, 1.2],
            "volume_fraction": 0.55,
            "centroid": [0.0, 0.0, 0.6],
            "triangle_count": 3200
          },
          {
            "name": "head",
            "bounds_min": [-0.1, -0.1, 1.5],
            "bounds_max": [0.1, 0.1, 1.78],
            "volume_fraction": 0.12,
            "centroid": [0.0, 0.0, 1.64],
            "triangle_count": 1200
          },
          {
            "name": "arm.L",
            "bounds_min": [-0.45, -0.06, 0.8],
            "bounds_max": [-0.18, 0.06, 1.4],
            "volume_fraction": 0.08,
            "centroid": [-0.32, 0.0, 1.1],
            "triangle_count": 800
          },
          {
            "name": "arm.R",
            "bounds_min": [0.18, -0.06, 0.8],
            "bounds_max": [0.45, 0.06, 1.4],
            "volume_fraction": 0.08,
            "centroid": [0.32, 0.0, 1.1],
            "triangle_count": 800
          },
          {
            "name": "leg.L",
            "bounds_min": [-0.15, -0.1, 0.0],
            "bounds_max": [-0.02, 0.1, 0.8],
            "volume_fraction": 0.085,
            "centroid": [-0.085, 0.0, 0.4],
            "triangle_count": 1200
          },
          {
            "name": "leg.R",
            "bounds_min": [0.02, -0.1, 0.0],
            "bounds_max": [0.15, 0.1, 0.8],
            "volume_fraction": 0.085,
            "centroid": [0.085, 0.0, 0.4],
            "triangle_count": 1200
          }
        ],
        "adjacency": [
          { "part_a": "body", "part_b": "head", "distance": -0.02 },
          { "part_a": "body", "part_b": "arm.L", "distance": 0.0 },
          { "part_a": "body", "part_b": "arm.R", "distance": 0.0 },
          { "part_a": "body", "part_b": "leg.L", "distance": 0.0 },
          { "part_a": "body", "part_b": "leg.R", "distance": 0.0 }
        ]
      },
      "skeletal": {
        "hierarchy_depth": 5,
        "terminal_bones": ["hand.L", "hand.R", "foot.L", "foot.R", "head"],
        "bone_coverage": [
          {
            "bone_name": "spine",
            "bone_length": 0.5,
            "mesh_length_along_bone": 0.52,
            "coverage_ratio": 1.04,
            "mesh_radius_avg": 0.14
          },
          {
            "bone_name": "upper_arm.L",
            "bone_length": 0.28,
            "mesh_length_along_bone": 0.26,
            "coverage_ratio": 0.93,
            "mesh_radius_avg": 0.048
          },
          {
            "bone_name": "upper_arm.R",
            "bone_length": 0.28,
            "mesh_length_along_bone": 0.26,
            "coverage_ratio": 0.93,
            "mesh_radius_avg": 0.049
          }
        ],
        "bone_symmetry": [
          {
            "bone_left": "upper_arm.L",
            "bone_right": "upper_arm.R",
            "length_ratio": 1.0,
            "radius_ratio": 0.98
          },
          {
            "bone_left": "thigh.L",
            "bone_right": "thigh.R",
            "length_ratio": 1.0,
            "radius_ratio": 1.01
          }
        ]
      },
      "scale": {
        "longest_dimension_m": 1.78,
        "volume_m3": 0.225,
        "fits_in_1m_cube": false,
        "fits_in_10cm_cube": false
      }
    }
  }
}
```

## Usage for LLMs

Structural metrics enable self-correction by comparing factual measurements against stated intent.

### Workflow

1. **State intent clearly** - Before generation, define expected properties:
   - "A 1.8m tall humanoid character"
   - "Bilaterally symmetric"
   - "Arms should be proportional to body"

2. **Generate the mesh** - Run the SpecCade pipeline

3. **Compare metrics to intent** - Check each metric against expectations:
   - `scale.longest_dimension_m` should be ~1.8
   - `symmetry.x_axis` should be > 0.9
   - `components` should show arm volume fractions similar to each other

4. **Identify discrepancies** - Look for mismatches:
   - If `elongation` is 2.0 instead of expected 6.0, the character is too wide
   - If `bone_symmetry[arm].radius_ratio` is 0.7, one arm is thinner

5. **Adjust and regenerate** - Modify the spec to correct issues

### Common Comparisons

| Intent | Metric to Check | Expected Range |
|--------|-----------------|----------------|
| "Tall and thin" | `elongation` | > 4.0 |
| "Compact/stocky" | `elongation` | 1.0 - 2.0 |
| "Bilaterally symmetric" | `symmetry.x_axis` | > 0.9 |
| "Human-scale" | `scale.longest_dimension_m` | 1.5 - 2.0 |
| "Handheld prop" | `scale.fits_in_10cm_cube` | true |
| "Complex silhouette" | `convex_hull_ratio` | < 0.5 |
| "Simple shape" | `convex_hull_ratio` | > 0.8 |
| "Centered mass" | `centroid_normalized` | All ~0.5 |
| "Top-heavy" | `centroid_normalized[2]` | > 0.6 |

### Error Detection Examples

**Problem:** Character appears as a blob instead of distinct humanoid shape

**Check:**
- `components.components` - Should have 5-7 distinct parts (body, head, arms, legs)
- `convex_hull_ratio` - Should be < 0.5 for a character with limbs

**Problem:** Arms are different sizes

**Check:**
- `skeletal.bone_symmetry` - Find arm entries, check `radius_ratio`
- `components` - Compare `volume_fraction` for arm.L vs arm.R

**Problem:** Character is too small/large

**Check:**
- `scale.longest_dimension_m` - Compare to intended height
- `geometry.extent` - Check Z dimension for standing characters

## See Also

- [Static Mesh Specs](mesh.md) - Static mesh generation parameters
- [Character Specs](character.md) - Skeletal mesh with armature parameters
- [Animation Specs](animation.md) - Skeletal animation parameters
