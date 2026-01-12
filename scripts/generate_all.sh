#!/bin/bash
# Generate all speccade golden specs
#
# Usage:
#   ./generate_all.sh [OUTPUT_DIR] [OPTIONS]
#
# Options:
#   --include-blender    Include Blender-based assets (static_mesh, skeletal_mesh, skeletal_animation)
#   --verbose            Show verbose output
#   --help               Show this help message
#
# Examples:
#   ./generate_all.sh                          # Generate to ./test-outputs
#   ./generate_all.sh ./my-outputs             # Generate to ./my-outputs
#   ./generate_all.sh --include-blender        # Include Blender assets
#

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Default values
OUTPUT_DIR="./test-outputs"
INCLUDE_BLENDER=false
VERBOSE=false
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SPECCADE_ROOT="$(dirname "$SCRIPT_DIR")"
SPEC_DIR="$SPECCADE_ROOT/golden/speccade/specs"

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --include-blender)
            INCLUDE_BLENDER=true
            shift
            ;;
        --verbose)
            VERBOSE=true
            shift
            ;;
        --help|-h)
            head -20 "$0" | tail -18
            exit 0
            ;;
        -*)
            echo -e "${RED}Unknown option: $1${NC}"
            exit 1
            ;;
        *)
            OUTPUT_DIR="$1"
            shift
            ;;
    esac
done

# Asset types to process (match golden corpus directories under golden/speccade/specs/)
ASSET_TYPES=("audio" "music" "texture")

if [ "$INCLUDE_BLENDER" = true ]; then
    ASSET_TYPES+=("static_mesh" "skeletal_mesh" "skeletal_animation")
fi

# Helper: count outputs in a spec file
count_outputs() {
    local spec_file="$1"
    # Count the number of entries in the spec's "outputs" array.
    #
    # Grepping `"path":` is brittle because other objects may also contain `"path"` keys.
    if command -v python3 >/dev/null 2>&1; then
        python3 - "$spec_file" <<'PY'
import json
import sys

with open(sys.argv[1], "r", encoding="utf-8") as f:
    data = json.load(f)

outputs = data.get("outputs") or []
print(len(outputs))
PY
    else
        grep -o '"path"[[:space:]]*:' "$spec_file" | wc -l | tr -d ' '
    fi
}

# Helper: get the extension from the first output path
get_output_extension() {
    local spec_file="$1"
    # Extract the first output path and return its file extension.
    if command -v python3 >/dev/null 2>&1; then
        python3 - "$spec_file" <<'PY'
import json
import os
import sys

with open(sys.argv[1], "r", encoding="utf-8") as f:
    data = json.load(f)

outputs = data.get("outputs") or []
if not outputs:
    print("")
    raise SystemExit(0)

path = outputs[0].get("path") or ""
_, ext = os.path.splitext(path)
print(ext.lstrip("."))
PY
    else
        grep -o '"path"[[:space:]]*:[[:space:]]*"[^"]*"' "$spec_file" | head -1 | sed 's/.*"\([^"]*\)".*/\1/' | sed 's/.*\.//'
    fi
}

# Statistics
TOTAL_SPECS=0
SUCCESS_COUNT=0
FAILURE_COUNT=0
SKIPPED_COUNT=0
declare -a FAILED_SPECS
declare -a SUCCESS_HASHES

# Start timer
START_TIME=$(date +%s)

echo -e "${CYAN}======================================${NC}"
echo -e "${CYAN}  SpecCade Golden Spec Generator${NC}"
echo -e "${CYAN}======================================${NC}"
echo ""
echo -e "${BLUE}Spec directory:${NC} $SPEC_DIR"
echo -e "${BLUE}Output directory:${NC} $OUTPUT_DIR"
echo -e "${BLUE}Include Blender:${NC} $INCLUDE_BLENDER"
echo ""

# Create output directories
mkdir -p "$OUTPUT_DIR"
for asset_type in "${ASSET_TYPES[@]}"; do
    mkdir -p "$OUTPUT_DIR/$asset_type"
done

# Build speccade-cli first
echo -e "${YELLOW}Building speccade-cli...${NC}"
cd "$SPECCADE_ROOT"
if cargo build -p speccade-cli --release 2>/dev/null; then
    echo -e "${GREEN}Build successful${NC}"
else
    echo -e "${RED}Build failed${NC}"
    exit 1
fi
echo ""

# Find the binary
SPECCADE_BIN="$SPECCADE_ROOT/target/release/speccade"
if [ ! -f "$SPECCADE_BIN" ]; then
    SPECCADE_BIN="$SPECCADE_ROOT/target/release/speccade.exe"
fi

if [ ! -f "$SPECCADE_BIN" ]; then
    echo -e "${RED}Could not find speccade binary${NC}"
    exit 1
fi

# Process a single spec file
# Arguments: $1 = spec path, $2 = output category
process_spec() {
    local spec="$1"
    local output_category="$2"

    TOTAL_SPECS=$((TOTAL_SPECS + 1))
    name=$(basename "$spec" .json)
    output_count=$(count_outputs "$spec")

    if [ "$VERBOSE" = true ]; then
        echo -e "  ${BLUE}Generating:${NC} $name (${output_count} output(s))"
    fi

    # Determine output location based on number of outputs
    if [ "$output_count" -eq 1 ]; then
        # Single output: write directly to category folder
        # Get the file extension from the spec
        ext=$(get_output_extension "$spec")
        out_file="$OUTPUT_DIR/$output_category/$name.$ext"
        out_dir="$OUTPUT_DIR/$output_category"
        mkdir -p "$out_dir"
        log_file="$out_dir/$name.generation.log"
    else
        # Multiple outputs: create subdirectory
        out_dir="$OUTPUT_DIR/$output_category/$name"
        mkdir -p "$out_dir"
        log_file="$out_dir/generation.log"
    fi

    # Run generation
    if "$SPECCADE_BIN" generate --spec "$spec" --out-root "$out_dir" > "$log_file" 2>&1; then
        SUCCESS_COUNT=$((SUCCESS_COUNT + 1))

        # Extract hash from report if available
        asset_id=$(grep -o '"asset_id"[[:space:]]*:[[:space:]]*"[^"]*"' "$spec" | head -1 | sed 's/.*"asset_id"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/')
        report_file="$(dirname "$spec")/${asset_id}.report.json"
        if [ -n "$report_file" ] && [ -f "$report_file" ]; then
            hash=$(grep -o '"spec_hash"[[:space:]]*:[[:space:]]*"[^"]*"' "$report_file" 2>/dev/null | cut -d'"' -f4 || echo "unknown")
            SUCCESS_HASHES+=("$output_category/$name: $hash")
        fi

        if [ "$VERBOSE" = true ]; then
            echo -e "    ${GREEN}SUCCESS${NC}"
        else
            echo -n -e "${GREEN}.${NC}"
        fi
    else
        FAILURE_COUNT=$((FAILURE_COUNT + 1))
        FAILED_SPECS+=("$output_category/$name")

        if [ "$VERBOSE" = true ]; then
            echo -e "    ${RED}FAILED${NC}"
            cat "$log_file" | head -10
        else
            echo -n -e "${RED}x${NC}"
        fi
    fi
}

# Process each asset type
for asset_type in "${ASSET_TYPES[@]}"; do
    ASSET_DIR="$SPEC_DIR/$asset_type"
    DIRS_TO_PROCESS=("$ASSET_DIR")

    has_specs=false
    for dir in "${DIRS_TO_PROCESS[@]}"; do
        if [ -d "$dir" ]; then
            for spec in "$dir"/*.json; do
                if [ -f "$spec" ]; then
                    has_specs=true
                    break 2
                fi
            done
        fi
    done

    if [ "$has_specs" = false ]; then
        echo -e "${YELLOW}Skipping $asset_type (no specs found)${NC}"
        continue
    fi

    echo -e "${CYAN}Processing $asset_type...${NC}"

    for dir in "${DIRS_TO_PROCESS[@]}"; do
        if [ ! -d "$dir" ]; then
            continue
        fi

        for spec in "$dir"/*.json; do
            if [ ! -f "$spec" ]; then
                continue
            fi

            process_spec "$spec" "$asset_type"
        done
    done

    if [ "$VERBOSE" = false ]; then
        echo ""  # Newline after dots
    fi
done

# Calculate elapsed time
END_TIME=$(date +%s)
ELAPSED=$((END_TIME - START_TIME))

# Print summary
echo ""
echo -e "${CYAN}======================================${NC}"
echo -e "${CYAN}  Generation Summary${NC}"
echo -e "${CYAN}======================================${NC}"
echo ""
echo -e "${BLUE}Total specs processed:${NC} $TOTAL_SPECS"
echo -e "${GREEN}Successful:${NC} $SUCCESS_COUNT"
echo -e "${RED}Failed:${NC} $FAILURE_COUNT"
echo -e "${YELLOW}Skipped:${NC} $SKIPPED_COUNT"
echo -e "${BLUE}Total runtime:${NC} ${ELAPSED}s"
echo ""

# List successful specs with hashes
if [ ${#SUCCESS_HASHES[@]} -gt 0 ]; then
    echo -e "${GREEN}Generated specs with BLAKE3 hashes:${NC}"
    for entry in "${SUCCESS_HASHES[@]}"; do
        echo -e "  $entry"
    done
    echo ""
fi

# List failed specs
if [ ${#FAILED_SPECS[@]} -gt 0 ]; then
    echo -e "${RED}Failed specs:${NC}"
    for spec in "${FAILED_SPECS[@]}"; do
        echo -e "  - $spec"
    done
    echo ""
fi

echo -e "${BLUE}Outputs saved to:${NC} $OUTPUT_DIR"

# Write summary report
SUMMARY_FILE="$OUTPUT_DIR/generation_summary.json"
cat > "$SUMMARY_FILE" << EOF
{
  "timestamp": "$(date -Iseconds)",
  "total_specs": $TOTAL_SPECS,
  "successful": $SUCCESS_COUNT,
  "failed": $FAILURE_COUNT,
  "skipped": $SKIPPED_COUNT,
  "runtime_seconds": $ELAPSED,
  "include_blender": $INCLUDE_BLENDER,
  "failed_specs": [$(printf '"%s",' "${FAILED_SPECS[@]}" | sed 's/,$//')]
}
EOF

echo -e "${BLUE}Summary report:${NC} $SUMMARY_FILE"

# Exit with error if any specs failed
if [ $FAILURE_COUNT -gt 0 ]; then
    exit 1
fi

exit 0
