#!/bin/bash
# SpecCade Asset Validation Examples
# 
# This script demonstrates common validation workflows for the SpecCade asset pipeline.
# 
# Prerequisites:
#   - speccade CLI installed (cargo install --path crates/speccade-cli)
#   - Valid spec files in the specs/ directory
#
# Usage:
#   chmod +x examples/validation/validation-example.sh
#   ./examples/validation/validation-example.sh

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}  SpecCade Validation Examples${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""

# Check if speccade is available
if ! command -v speccade &> /dev/null; then
    echo -e "${YELLOW}Warning: speccade not found in PATH${NC}"
    echo "Installing from source..."
    cargo install --path crates/speccade-cli --quiet
fi

# Configuration
SPECS_DIR="${SPECS_DIR:-specs}"
OUTPUT_DIR="${OUTPUT_DIR:-validation-examples-output}"

# ============================================================================
# Example 1: Single Asset Validation
# ============================================================================

echo -e "${BLUE}Example 1: Single Asset Validation${NC}"
echo "----------------------------------------"

# Find a mesh spec to validate
MESH_SPEC=$(find "$SPECS_DIR" -name "*.star" -path "*/mesh/*" | head -1)

if [ -n "$MESH_SPEC" ]; then
    echo "Validating: $MESH_SPEC"
    
    # Run validation with custom output directory
    speccade validate-asset \
        --spec "$MESH_SPEC" \
        --out-root "$OUTPUT_DIR/single"
    
    echo -e "${GREEN}✓ Single validation complete${NC}"
    echo "  Output: $OUTPUT_DIR/single/"
    echo ""
else
    echo -e "${YELLOW}No mesh specs found in $SPECS_DIR/mesh/${NC}"
    echo "  Skipping single asset validation example"
    echo ""
fi

# ============================================================================
# Example 2: Batch Validation
# ============================================================================

echo -e "${BLUE}Example 2: Batch Validation${NC}"
echo "----------------------------------------"

# Count available specs
MESH_COUNT=$(find "$SPECS_DIR" -name "*.star" -path "*/mesh/*" 2>/dev/null | wc -l)

if [ "$MESH_COUNT" -gt 0 ]; then
    echo "Found $MESH_COUNT mesh specs"
    echo "Running batch validation..."
    
    # Validate all mesh specs
    speccade batch-validate \
        --specs "$SPECS_DIR/mesh/*.star" \
        --out-root "$OUTPUT_DIR/batch"
    
    echo -e "${GREEN}✓ Batch validation complete${NC}"
    echo "  Report: $OUTPUT_DIR/batch/batch-report.json"
    echo ""
    
    # Display summary if jq is available
    if command -v jq &> /dev/null && [ -f "$OUTPUT_DIR/batch/batch-report.json" ]; then
        echo "Batch Summary:"
        jq -r '"  Total: \(.total), Passed: \(.passed), Failed: \(.failed)"' \
            "$OUTPUT_DIR/batch/batch-report.json"
        echo ""
    fi
else
    echo -e "${YELLOW}No mesh specs available for batch validation${NC}"
    echo ""
fi

# ============================================================================
# Example 3: Validate with Validation Comments
# ============================================================================

echo -e "${BLUE}Example 3: Extracting Validation Comments${NC}"
echo "----------------------------------------"

# Create a sample spec with validation comments
SAMPLE_SPEC="$OUTPUT_DIR/sample_with_comments.star"

cat > "$SAMPLE_SPEC" << 'EOF'
# [VALIDATION]
# SHAPE: Sample validation cube
# PROPORTIONS: 1.0 x 1.0 x 1.0 units
# ORIENTATION: Centered at origin
# FRONT VIEW: Square face visible
# NOTES: Example spec for validation guide

spec(
    asset_id = "examples/validation-cube",
    asset_type = "static_mesh",
    license = "CC0-1.0",
    seed = 42,
    recipe = mesh.primitives(
        base_primitive = "cube",
        dimensions = [1.0, 1.0, 1.0],
    ),
    outputs = [
        output(
            kind = "primary",
            format = "glb",
            path = "validation-cube.glb",
        ),
    ],
)
EOF

echo "Created sample spec with validation comments"
echo "  Location: $SAMPLE_SPEC"
echo ""
echo "Validation comments in spec:"
grep -A 20 "# \[VALIDATION\]" "$SAMPLE_SPEC" | head -15
echo ""

# ============================================================================
# Example 4: Quality Gate Checking
# ============================================================================

echo -e "${BLUE}Example 4: Checking Quality Gates${NC}"
echo "----------------------------------------"

# If we have a validation report, check quality gates
REPORT=$(find "$OUTPUT_DIR" -name "*.validation-report.json" | head -1)

if [ -n "$REPORT" ] && command -v jq &> /dev/null; then
    echo "Checking quality gates from: $(basename "$REPORT")"
    echo ""
    
    # Extract and display quality gates
    jq -r '
        .quality_gates | 
        to_entries | 
        .[] | 
        "  \(.key): \(.value | if . then "✓ PASS" else "✗ FAIL" end)"
    ' "$REPORT"
    
    # Check for lint errors
    ERROR_COUNT=$(jq '[.lint_results[] | select(.severity == "error")] | length' "$REPORT")
    
    if [ "$ERROR_COUNT" -gt 0 ]; then
        echo ""
        echo -e "${RED}  ⚠ Found $ERROR_COUNT lint error(s)${NC}"
        jq -r '.lint_results[] | select(.severity == "error") | "    - \(.rule_id): \(.message)"' "$REPORT"
    else
        echo -e "${GREEN}  ✓ No lint errors${NC}"
    fi
    
    echo ""
else
    echo -e "${YELLOW}No validation report found or jq not installed${NC}"
    echo "  Install jq to parse JSON reports: https://stedolan.github.io/jq/"
    echo ""
fi

# ============================================================================
# Example 5: CI/CD Integration Pattern
# ============================================================================

echo -e "${BLUE}Example 5: CI/CD Integration Pattern${NC}"
echo "----------------------------------------"

cat << 'EOF'
This is how you would integrate validation in a CI pipeline:

# .github/workflows/validate-assets.yml
name: Asset Validation

on:
  push:
    paths: ['specs/**/*.star']

jobs:
  validate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Install SpecCade
        run: cargo install --path crates/speccade-cli
      - name: Validate Assets
        run: |
          failed=0
          for spec in specs/**/*.star; do
            speccade validate-asset --spec "$spec" || failed=1
          done
          exit $failed

Key points:
- Trigger on spec file changes only
- Fail the build if validation fails
- Install fresh speccade on each run
EOF

echo ""

# ============================================================================
# Example 6: Parse Batch Report
# ============================================================================

echo -e "${BLUE}Example 6: Parsing Batch Validation Results${NC}"
echo "----------------------------------------"

BATCH_REPORT="$OUTPUT_DIR/batch/batch-report.json"

if [ -f "$BATCH_REPORT" ] && command -v jq &> /dev/null; then
    echo "Batch Report Analysis:"
    echo ""
    
    # Show failed validations
    FAILED=$(jq -r '.results[] | select(.success == false) | .spec' "$BATCH_REPORT")
    if [ -n "$FAILED" ]; then
        echo -e "${RED}Failed validations:${NC}"
        echo "$FAILED" | while read -r spec; do
            echo "  - $spec"
        done
    else
        echo -e "${GREEN}✓ All validations passed${NC}"
    fi
    
    echo ""
    
    # Show summary statistics
    echo "Statistics:"
    jq -r '
        "  Total specs: \(.total)",
        "  Passed: \(.passed)",
        "  Failed: \(.failed)",
        "  Success rate: \(.passed / .total * 100 | floor)%"
    ' "$BATCH_REPORT"
    
    echo ""
else
    echo -e "${YELLOW}No batch report available${NC}"
    echo ""
fi

# ============================================================================
# Summary
# ============================================================================

echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}  Examples Complete${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""
echo "Output directory: $OUTPUT_DIR/"
echo ""
echo "Next steps:"
echo "  1. Review validation reports in $OUTPUT_DIR/"
echo "  2. Check preview grids (*.grid.png) for visual verification"
echo "  3. Inspect metrics in *.metrics.json files"
echo "  4. Review lint results in *.lint.json files"
echo ""
echo "Documentation:"
echo "  - Full guide: docs/validation-guide.md"
echo "  - CLI help: speccade validate-asset --help"
echo "  - Batch help: speccade batch-validate --help"
