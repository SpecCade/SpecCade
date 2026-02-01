# SpecCade Asset Validation Guide

A comprehensive guide to using the `validate-asset` and `batch-validate` commands for automated asset quality assurance.

## Table of Contents

1. [Quick Start](#quick-start)
2. [Understanding Validation Reports](#understanding-validation-reports)
3. [Validation Comments in Specs](#validation-comments-in-specs)
4. [CI/CD Integration](#cicd-integration)
5. [Advanced Usage](#advanced-usage)

---

## Quick Start

### Single Asset Validation

Validate a single 3D asset (static mesh, skeletal mesh, or animation):

```bash
# Basic validation
speccade validate-asset --spec specs/mesh/cube.star

# With custom output directory
speccade validate-asset --spec specs/mesh/cube.star --out-root ./validation-output

# Using cargo (if not installed)
cargo run -p speccade-cli -- validate-asset --spec specs/mesh/cube.star
```

The command will:
1. Generate the GLB asset from the spec
2. Create a preview grid PNG showing 6 different angles
3. Extract and analyze mesh metrics (vertex count, UVs, skeleton, etc.)
4. Run lint rules against the asset
5. Generate a comprehensive JSON report

### Batch Validation

Validate multiple assets at once:

```bash
# Validate all mesh specs
speccade batch-validate --specs "specs/mesh/*.star"

# Validate all 3D assets recursively
speccade batch-validate --specs "specs/**/*.{star,json}" --out-root ./batch-results
```

---

## Understanding Validation Reports

### Report Structure

Each validation produces a JSON report with the following structure:

```json
{
  "spec_path": "specs/mesh/cube.star",
  "asset_id": "mesh-cube-01",
  "asset_type": "static_mesh",
  "timestamp": "2025-02-01T10:30:00Z",
  "generation": {
    "success": true,
    "asset_path": "validation-output/mesh-cube-01.glb",
    "error": null
  },
  "visual_evidence": {
    "grid_path": "mesh-cube-01.grid.png",
    "grid_generated": true
  },
  "metrics": {
    "topology": {
      "vertex_count": 24,
      "triangle_count": 12,
      "edge_count": 36
    },
    "manifold": {
      "manifold": true,
      "boundary_edges": 0,
      "non_manifold_edges": 0
    },
    "uv": {
      "has_uvs": true,
      "uv_islands": 6,
      "uv_coverage_percent": 98.5
    }
  },
  "lint_results": [
    {
      "rule_id": "mesh.uv_bounds",
      "severity": "warning",
      "message": "UV coordinates exceed [0,1] range",
      "category": "uv"
    }
  ],
  "quality_gates": {
    "generation": true,
    "has_geometry": true,
    "manifold": true,
    "has_uvs": true,
    "skeleton_valid": false,
    "animation_valid": false
  },
  "validation_comments": "SHAPE: A beveled cube with smooth subdivision..."
}
```

### Quality Gates

Quality gates are boolean checks that determine if the asset meets minimum requirements:

| Gate | Description | Applies To |
|------|-------------|------------|
| `generation` | Asset was generated successfully | All |
| `has_geometry` | Mesh has vertices and triangles | All meshes |
| `manifold` | Mesh is manifold (no holes, consistent winding) | All meshes |
| `has_uvs` | Mesh has UV coordinates | All meshes |
| `skeleton_valid` | Skeleton has at least one bone | Skeletal meshes |
| `animation_valid` | Animation has at least one clip | Animations |

### Visual Evidence

The preview grid (`*.grid.png`) shows your asset from 6 angles:
- **Front** - Primary view
- **Back** - Opposite front
- **Top** - Bird's eye view
- **Left** - Left side profile
- **Right** - Right side profile
- **ISO** - Isometric perspective

Use this to visually verify proportions, orientation, and overall shape.

### Metrics

The metrics section provides quantitative data about your asset:

**Topology Metrics:**
- `vertex_count` - Number of vertices
- `triangle_count` - Number of triangles
- `edge_count` - Number of edges

**Manifold Metrics:**
- `manifold` - Whether the mesh is manifold
- `boundary_edges` - Edges on mesh boundaries
- `non_manifold_edges` - Edges shared by more than 2 faces

**UV Metrics:**
- `has_uvs` - Whether UVs are present
- `uv_islands` - Number of UV islands
- `uv_coverage_percent` - Percentage of UV space used

**Skeleton Metrics** (skeletal meshes only):
- `bone_count` - Number of bones in skeleton
- `max_bone_depth` - Maximum depth of bone hierarchy

**Animation Metrics** (animations only):
- `animation_count` - Number of animation clips
- `total_duration_seconds` - Total duration of all clips
- `frames_per_second` - Sample rate of animation data

---

## Validation Comments in Specs

### Purpose

Validation comments are human-readable descriptions embedded in Starlark specs that document:
- Expected shape and proportions
- Key visual characteristics
- Orientation requirements
- Special notes for reviewers

### Syntax

Add a `[VALIDATION]` block at the top of your `.star` file:

```starlark
# [VALIDATION]
# SHAPE: A beveled cube with smooth subdivision
# PROPORTIONS: Equal 1.0 unit dimensions on all axes
# ORIENTATION: Cube centered at origin
# FRONT VIEW: Square face visible, beveled edges
# TOP VIEW: Square face visible
# NOTES: Bevel creates smooth edges suitable for game assets

spec(
    asset_id = "mesh/cube-01",
    asset_type = "static_mesh",
    ...
)
```

### Guidelines

**Effective validation comments:**
- Describe the expected outcome, not the implementation
- Mention proportions and scale
- Note orientation (which direction is "front")
- Include any special visual features
- Keep under 10 lines for readability

**Example for a character model:**

```starlark
# [VALIDATION]
# SHAPE: Humanoid warrior with exaggerated proportions
# PROPORTIONS: Head 1:8 body ratio (heroic scale)
# ORIENTATION: Facing +Y axis, front is -Z
# KEY FEATURES: Pauldrons 1.5x shoulder width, sword on right hip
# FRONT VIEW: Symmetrical chest armor, face visible
# SIDE VIEW: Sword sheathed at hip, cape flowing back
# NOTES: T-pose for rigging, fingers slightly curled
```

**Example for an animation:**

```starlark
# [VALIDATION]
# ANIMATION: Idle breathing loop
# DURATION: 2.0 seconds, seamless loop
# MOVEMENT: Chest expands/contracts, subtle head bob
# ROOT MOTION: None (in-place)
# NOTES: Subtle enough to layer with other animations
```

### Integration

Validation comments are automatically extracted during validation and included in the JSON report under the `validation_comments` field. This allows:
- Automated checking against actual output
- Documentation in version control
- Human review during asset approval

---

## CI/CD Integration

### GitHub Actions

Add validation to your CI pipeline:

```yaml
# .github/workflows/asset-validation.yml
name: Asset Validation

on:
  push:
    paths:
      - 'specs/**/*.star'
      - 'specs/**/*.json'
  pull_request:
    paths:
      - 'specs/**/*.star'
      - 'specs/**/*.json'

jobs:
  validate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Install Rust
        uses: dtolnay/rust-action@stable
      
      - name: Install SpecCade
        run: cargo install --path crates/speccade-cli
      
      - name: Validate changed assets
        run: |
          failed=0
          for spec in $(git diff --name-only HEAD~1 | grep -E '\.(star|json)$'); do
            echo "Validating: $spec"
            if ! speccade validate-asset --spec "$spec" --out-root ./validation-results; then
              failed=1
            fi
          done
          exit $failed
      
      - name: Upload validation reports
        if: always()
        uses: actions/upload-artifact@v4
        with:
          name: validation-reports
          path: ./validation-results/**/*.validation-report.json
```

### GitLab CI

```yaml
# .gitlab-ci.yml
validate-assets:
  stage: test
  image: rust:latest
  script:
    - cargo install --path crates/speccade-cli
    - |
      for spec in $(git diff --name-only HEAD~1 | grep -E '\.(star|json)$'); do
        speccade validate-asset --spec "$spec" --out-root ./validation-results
      done
  artifacts:
    when: always()
    paths:
      - validation-results/
    expire_in: 1 week
  only:
    changes:
      - specs/**/*
```

### Pre-commit Hook

Validate assets before committing:

```bash
#!/bin/bash
# .git/hooks/pre-commit

# Get list of staged spec files
staged_specs=$(git diff --cached --name-only --diff-filter=ACM | grep -E '\.(star|json)$')

if [ -z "$staged_specs" ]; then
    exit 0
fi

echo "Running asset validation..."
failed=0

for spec in $staged_specs; do
    if ! speccade validate-asset --spec "$spec" --out-root .tmp-validation; then
        failed=1
    fi
done

# Cleanup
rm -rf .tmp-validation

if [ $failed -ne 0 ]; then
    echo "Validation failed! Fix issues before committing."
    exit 1
fi
```

### Quality Gates as Assertions

Parse the validation report to enforce quality gates:

```python
#!/usr/bin/env python3
# validate_and_check.py

import json
import sys
import subprocess

def validate_and_check(spec_path):
    """Validate an asset and check quality gates."""
    
    # Run validation
    result = subprocess.run(
        ['speccade', 'validate-asset', '--spec', spec_path, '--out-root', './validation'],
        capture_output=True,
        text=True
    )
    
    if result.returncode != 0:
        print(f"❌ Validation failed for {spec_path}")
        return False
    
    # Parse report
    report_path = f"./validation/{spec_path.replace('/', '_')}.validation-report.json"
    with open(report_path) as f:
        report = json.load(f)
    
    gates = report['quality_gates']
    passed = True
    
    # Required gates for all meshes
    required = ['generation', 'has_geometry', 'manifold', 'has_uvs']
    
    for gate in required:
        if not gates.get(gate):
            print(f"❌ Quality gate failed: {gate}")
            passed = False
        else:
            print(f"✓ Quality gate passed: {gate}")
    
    # Check for errors in lint
    errors = [r for r in report['lint_results'] if r['severity'] == 'error']
    if errors:
        print(f"❌ {len(errors)} lint errors found")
        for err in errors:
            print(f"  - [{err['rule_id']}] {err['message']}")
        passed = False
    
    return passed

if __name__ == '__main__':
    spec = sys.argv[1]
    success = validate_and_check(spec)
    sys.exit(0 if success else 1)
```

---

## Advanced Usage

### Custom Output Organization

Organize validation results by asset type:

```bash
#!/bin/bash
# validate-by-type.sh

ASSET_TYPES=("static_mesh" "skeletal_mesh" "skeletal_animation")

for type in "${ASSET_TYPES[@]}"; do
    echo "Validating $type assets..."
    mkdir -p "validation/$type"
    
    for spec in specs/$type/*.star; do
        if [ -f "$spec" ]; then
            speccade validate-asset \
                --spec "$spec" \
                --out-root "validation/$type"
        fi
    done
done
```

### Generating Validation Dashboards

Combine multiple validation reports into a summary:

```bash
#!/bin/bash
# generate-dashboard.sh

out_dir="validation-dashboard"
mkdir -p "$out_dir"

# Collect all reports
find . -name "*.validation-report.json" -exec cp {} "$out_dir/" \;

# Generate HTML summary
cat > "$out_dir/index.html" << 'EOF'
<!DOCTYPE html>
<html>
<head>
    <title>Asset Validation Dashboard</title>
    <style>
        body { font-family: system-ui, sans-serif; margin: 2rem; }
        .pass { color: green; }
        .fail { color: red; }
        table { border-collapse: collapse; width: 100%; }
        th, td { border: 1px solid #ddd; padding: 0.5rem; text-align: left; }
        th { background: #f5f5f5; }
    </style>
</head>
<body>
    <h1>Asset Validation Dashboard</h1>
    <table id="results">
        <tr>
            <th>Asset ID</th>
            <th>Type</th>
            <th>Generation</th>
            <th>Geometry</th>
            <th>Manifold</th>
            <th>UVs</th>
            <th>Report</th>
        </tr>
    </table>
    <script>
        // Load and display reports
        const reports = [];
        // ... fetch and populate table
    </script>
</body>
</html>
EOF

echo "Dashboard generated: $out_dir/index.html"
```

### Filtering Validation Results

Filter batch validation results to find failing assets:

```bash
# Find all failed validations
jq -r 'select(.success == false) | .spec' batch-validation/batch-report.json

# Find assets with UV warnings
jq -r 'select(.lint_results[]?.category == "uv") | .spec' batch-validation/batch-report.json

# Generate CSV of quality gates
cat batch-validation/batch-report.json | jq -r '
    .results[] | 
    select(.success == true) |
    .spec as $spec |
    "\($spec),\(.quality_gates | to_entries | map(.value) | join(","))"
'
```

---

## Troubleshooting

### Blender Not Found

If you see errors about Blender not being found:

```bash
# Check Blender installation
speccade doctor

# Set Blender path manually
export BLENDER_PATH=/usr/bin/blender
```

### Preview Grid Failures

If preview grid generation fails but the asset is valid:
- Check that the asset has valid geometry
- Verify the asset type is supported (static_mesh, skeletal_mesh, skeletal_animation)
- Check Blender logs in the output directory

### Memory Issues

For large batch validations, you may run out of memory:

```bash
# Validate one at a time instead of parallel
for spec in specs/mesh/*.star; do
    speccade validate-asset --spec "$spec"
done
```

---

## Implementation Summary

This validation system was implemented with the following components:

1. **validate-asset command** (`crates/speccade-cli/src/commands/validate_asset.rs`)
   - Runs full validation pipeline: generate → preview → analyze → lint
   - Produces comprehensive JSON reports with quality gates
   - Extracts validation comments from Starlark specs

2. **batch-validate command** (`crates/speccade-cli/src/commands/batch_validate.rs`)
   - Validates multiple assets in sequence
   - Generates consolidated batch reports
   - Progress indicators and timeout handling

3. **Integration tests** (`crates/speccade-tests/tests/validate_asset_integration.rs`)
   - Tests validate-asset with static mesh
   - Tests validation rejection of non-3D assets
   - Verifies report structure and quality gates

4. **Documentation**
   - This validation guide
   - Example shell scripts for CI/CD integration
   - Troubleshooting and best practices

---

**Last Updated:** 2025-02-01

For the latest documentation, see the [SpecCade repository](https://github.com/your-org/speccade).
