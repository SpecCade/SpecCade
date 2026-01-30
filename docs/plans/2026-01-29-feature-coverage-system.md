# Plan: CI-Verifiable Feature Coverage System

**Date:** 2026-01-29
**Status:** Approved for implementation
**Scope:** Full automation - CLI command + generated YAML + CI enforcement

## Goal

Create a **fully automated** system that ensures 100% feature coverage in golden examples. The system must:

1. **Auto-discover** all features from source (stdlib, schemas, enums)
2. **Auto-scan** golden examples to find coverage
3. **Auto-generate** a machine-readable coverage report
4. **Auto-enforce** 100% coverage in CI (no allowlist, no exceptions)

## Core Principle: No Manual Maintenance

**The coverage document is GENERATED, not maintained by hand.**

- Features are discovered by parsing `stdlib.snapshot.json` and recipe schemas
- Coverage is detected by scanning `golden/` files for function calls and recipe fields
- The YAML output is regenerated on every CI run
- Humans only create golden examples; they never edit coverage tracking

---

## Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                        SOURCE OF TRUTH                               │
├─────────────────────────────────────────────────────────────────────┤
│  golden/stdlib/stdlib.snapshot.json    (all stdlib functions)       │
│  crates/speccade-core/src/recipe/*.rs  (all recipe types/enums)     │
└─────────────────────────────────────────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────────────┐
│                    speccade coverage generate                        │
│  1. Parse stdlib.snapshot.json → list of functions + enums          │
│  2. Parse recipe schemas → synthesis types, node types, etc.        │
│  3. Scan golden/starlark/*.star → find function usages              │
│  4. Scan golden/speccade/specs/**/*.json → find recipe features     │
│  5. Generate docs/coverage/feature-coverage.yaml                     │
└─────────────────────────────────────────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────────────┐
│                         CI ENFORCEMENT                               │
│  cargo test -p speccade-tests --test feature_coverage               │
│  • Loads generated YAML                                              │
│  • Asserts coverage_percent == 100.0                                 │
│  • Asserts uncovered == 0                                            │
│  • BLOCKS merge if any feature lacks an example                      │
└─────────────────────────────────────────────────────────────────────┘
```

---

## Implementation Phases

### Phase 1: Coverage Generator Command

**File:** `crates/speccade-cli/src/commands/coverage.rs`

```rust
/// Subcommand: speccade coverage
pub enum CoverageCommand {
    /// Generate coverage report (writes YAML)
    Generate {
        /// Fail if coverage < 100% (CI mode)
        #[arg(long)]
        strict: bool,
    },
    /// Print coverage summary to stdout
    Report,
}
```

**Generator Logic:**

```rust
fn generate_coverage() -> CoverageReport {
    // 1. Load feature inventory
    let stdlib = parse_stdlib_snapshot("golden/stdlib/stdlib.snapshot.json");
    let recipes = discover_recipe_features();  // from schema introspection

    // 2. Scan golden examples
    let starlark_usages = scan_starlark_files("golden/starlark/**/*.star");
    let spec_usages = scan_json_specs("golden/speccade/specs/**/*.json");

    // 3. Build coverage map
    let mut coverage = CoverageReport::new();

    for func in &stdlib.functions {
        let examples = find_usages(&func.name, &starlark_usages, &spec_usages);
        coverage.add_function(&func, examples);
    }

    for (enum_name, values) in &stdlib.enums {
        for value in values {
            let examples = find_enum_usage(enum_name, value, &starlark_usages, &spec_usages);
            coverage.add_enum_value(enum_name, value, examples);
        }
    }

    // 4. Recipe-level features
    for recipe_type in &recipes.synthesis_types {
        let examples = find_synthesis_type(recipe_type, &spec_usages);
        coverage.add_recipe_feature("synthesis", recipe_type, examples);
    }

    coverage
}
```

**Output Format:** `docs/coverage/feature-coverage.yaml`

```yaml
# AUTO-GENERATED - Do not edit manually
# Regenerate with: speccade coverage generate
schema_version: 1
generated_at: "2026-01-29T12:00:00Z"

summary:
  total_features: 247
  covered: 247
  uncovered: 0
  coverage_percent: 100.0

stdlib:
  audio.synthesis:
    - name: oscillator
      covered: true
      examples:
        - golden/starlark/audio_synth_oscillator.star:15
        - golden/speccade/specs/audio/simple_beep.json
      enum_coverage:
        waveform:
          sine: { covered: true, example: "golden/starlark/audio_synth_oscillator.star:15" }
          square: { covered: true, example: "golden/starlark/audio_synth_oscillator.star:23" }
          sawtooth: { covered: true, example: "golden/starlark/audio_synth_fm.star:8" }
          triangle: { covered: true, example: "golden/speccade/specs/audio/triangle_arp.json" }
          pulse: { covered: true, example: "golden/starlark/audio_synth_pulse.star:10" }

    - name: fm_synth
      covered: true
      examples:
        - golden/starlark/audio_synth_fm.star:1
        - golden/speccade/specs/audio/fm_bell.json

  # ... all other categories auto-populated ...

recipes:
  audio_v1:
    synthesis_types:
      oscillator: { covered: true, examples: ["golden/speccade/specs/audio/simple_beep.json"] }
      fm_synth: { covered: true, examples: ["golden/speccade/specs/audio/fm_bell.json"] }
      karplus_strong: { covered: true, examples: ["golden/speccade/specs/audio/pluck_string.json"] }
      # ... all types auto-discovered and checked ...

  texture.procedural_v1:
    node_types:
      noise: { covered: true, examples: ["golden/speccade/specs/texture/noise_fbm.json"] }
      gradient: { covered: true, examples: ["golden/starlark/texture_patterns.star"] }
      # ... all node types ...

# If any feature is uncovered, it appears here (CI blocks on non-empty)
uncovered_features: []
```

### Phase 2: CI Test Enforcement

**File:** `crates/speccade-tests/tests/feature_coverage.rs`

```rust
//! Feature coverage enforcement tests
//!
//! Policy: 100% coverage required. No allowlist. No exceptions.
//! If a feature exists, it MUST have a golden example.

use speccade_cli::coverage::{generate_coverage, CoverageReport};

#[test]
fn coverage_is_100_percent() {
    let report = generate_coverage();

    assert_eq!(
        report.summary.uncovered, 0,
        "BLOCKING: {} features have no golden examples.\n\
         Run `speccade coverage report` to see which features need examples.\n\
         Create examples in golden/starlark/ or golden/speccade/specs/.\n\n\
         Uncovered features:\n{}",
        report.summary.uncovered,
        report.format_uncovered_list()
    );

    assert_eq!(
        report.summary.coverage_percent, 100.0,
        "Coverage must be exactly 100%. Current: {:.1}%",
        report.summary.coverage_percent
    );
}

#[test]
fn all_enum_values_covered() {
    let report = generate_coverage();

    let uncovered_enums: Vec<_> = report.enums
        .iter()
        .flat_map(|(name, values)| {
            values.iter()
                .filter(|(_, info)| !info.covered)
                .map(move |(value, _)| format!("{}::{}", name, value))
        })
        .collect();

    assert!(
        uncovered_enums.is_empty(),
        "BLOCKING: {} enum values have no examples:\n  {}",
        uncovered_enums.len(),
        uncovered_enums.join("\n  ")
    );
}

#[test]
fn all_synthesis_types_covered() {
    let report = generate_coverage();

    let uncovered: Vec<_> = report.recipes.synthesis_types
        .iter()
        .filter(|(_, info)| !info.covered)
        .map(|(name, _)| name.as_str())
        .collect();

    assert!(
        uncovered.is_empty(),
        "BLOCKING: {} synthesis types have no examples: {:?}",
        uncovered.len(),
        uncovered
    );
}

#[test]
fn all_texture_nodes_covered() {
    let report = generate_coverage();

    let uncovered: Vec<_> = report.recipes.texture_nodes
        .iter()
        .filter(|(_, info)| !info.covered)
        .map(|(name, _)| name.as_str())
        .collect();

    assert!(
        uncovered.is_empty(),
        "BLOCKING: {} texture node types have no examples: {:?}",
        uncovered.len(),
        uncovered
    );
}

#[test]
fn all_mesh_primitives_covered() {
    let report = generate_coverage();

    let uncovered: Vec<_> = report.recipes.mesh_primitives
        .iter()
        .filter(|(_, info)| !info.covered)
        .map(|(name, _)| name.as_str())
        .collect();

    assert!(
        uncovered.is_empty(),
        "BLOCKING: {} mesh primitives have no examples: {:?}",
        uncovered.len(),
        uncovered
    );
}

#[test]
fn coverage_yaml_is_fresh() {
    // Regenerate coverage and compare to committed file
    let fresh = generate_coverage();
    let committed = load_yaml("docs/coverage/feature-coverage.yaml");

    assert_eq!(
        fresh.summary, committed.summary,
        "Coverage YAML is stale. Run `speccade coverage generate` and commit the result."
    );
}
```

**CRITICAL: Remove existing allowlist from `stdlib_coverage.rs`**

```rust
// BEFORE (remove this)
const ALLOWLIST: &[&str] = &["mesh_primitive"];

// AFTER (no exceptions)
// No allowlist. Every function must have coverage.
```

### Phase 3: Stdlib Snapshot Enhancement

Enhance `stdlib.snapshot.json` to include enum definitions for complete tracking.

**File:** `golden/stdlib/stdlib.snapshot.json` (enhanced structure)

```json
{
  "version": "1.0",
  "generated_at": "2026-01-29T00:00:00Z",
  "functions": [
    {
      "name": "oscillator",
      "category": "audio.synthesis",
      "params": ["frequency", "waveform", "amplitude"],
      "description": "Creates a basic oscillator synthesis node"
    }
  ],
  "enums": {
    "waveform": ["sine", "square", "sawtooth", "triangle", "pulse"],
    "noise_type": ["white", "pink", "brown", "blue"],
    "filter_type": ["lowpass", "highpass", "bandpass", "notch", "allpass"],
    "mesh_primitive": ["cube", "sphere", "cylinder", "cone", "torus", "plane", "ico_sphere"]
  },
  "recipe_types": {
    "synthesis": ["oscillator", "fm_synth", "karplus_strong", "noise", "additive", "modal", "granular", "wavetable", "vocoder", "formant"],
    "texture_node": ["noise", "gradient", "constant", "threshold", "invert", "add", "multiply", "lerp", "clamp", "step", "smoothstep"],
    "effect": ["reverb", "delay", "chorus", "phaser", "flanger", "bitcrush", "distortion", "compressor", "limiter", "eq"]
  }
}
```

### Phase 4: CI Workflow Update

**File:** `.github/workflows/ci.yml`

```yaml
jobs:
  test:
    steps:
      - name: Feature coverage check
        run: |
          # Generate fresh coverage report
          cargo run -p speccade-cli -- coverage generate --strict

          # Run coverage tests (blocks on any gap)
          cargo test -p speccade-tests --test feature_coverage -- --nocapture
```

---

## CLI Commands

```bash
# Generate coverage YAML (overwrites docs/coverage/feature-coverage.yaml)
speccade coverage generate

# Generate with strict mode - exits 1 if coverage < 100%
speccade coverage generate --strict

# Print human-readable coverage report to stdout
speccade coverage report

# Example output of report:
# Feature Coverage Report
# =======================
# Total features: 247
# Covered: 247 (100.0%)
# Uncovered: 0
#
# By Category:
#   audio.synthesis: 23/23 (100%)
#   audio.envelopes: 8/8 (100%)
#   audio.filters: 12/12 (100%)
#   ...
```

---

## Files to Create/Modify

| File | Action | Purpose |
|------|--------|---------|
| `crates/speccade-cli/src/commands/coverage.rs` | CREATE | Coverage generator command |
| `crates/speccade-cli/src/commands/mod.rs` | MODIFY | Register coverage subcommand |
| `crates/speccade-cli/src/main.rs` | MODIFY | Add coverage to CLI |
| `crates/speccade-tests/tests/feature_coverage.rs` | CREATE | CI enforcement tests |
| `crates/speccade-tests/tests/stdlib_coverage.rs` | MODIFY | Remove ALLOWLIST, integrate |
| `docs/coverage/feature-coverage.yaml` | CREATE | Generated coverage document |
| `golden/stdlib/stdlib.snapshot.json` | MODIFY | Add enums and recipe_types |
| `.github/workflows/ci.yml` | MODIFY | Add coverage step |

---

## Verification Checklist

After implementation, verify:

1. **Generate works:** `speccade coverage generate` produces valid YAML
2. **Report works:** `speccade coverage report` shows readable output
3. **CI mode works:** `speccade coverage generate --strict` exits 0 when 100%, 1 otherwise
4. **Tests pass:** `cargo test -p speccade-tests --test feature_coverage`
5. **Failure detection:** Remove an example, verify tests fail
6. **New feature detection:** Add fake function to stdlib, verify it shows uncovered
7. **No allowlist:** Confirm `stdlib_coverage.rs` has no ALLOWLIST constant

---

## Handling Current Gaps

Before the strict CI can pass, any currently uncovered features need examples:

1. Run: `cargo test -p speccade-tests --test stdlib_coverage -- --nocapture`
2. Note any functions in ALLOWLIST (currently: `mesh_primitive`)
3. Create golden examples for each uncovered feature
4. Remove ALLOWLIST
5. Verify all tests pass

**Required new golden files (based on current ALLOWLIST):**
- `golden/starlark/mesh_primitives.star` - Uses `mesh_primitive()` with all primitive types

---

## Design Decisions

### Why generated YAML instead of manual?
- **Humans are bad at maintenance** - Manual tracking always drifts
- **Source already exists** - `stdlib.snapshot.json` defines all features
- **Deterministic** - Same inputs always produce same output
- **No merge conflicts** - Regenerate, don't merge

### Why 100% with no exceptions?
- **Exceptions grow** - Today's "temporary" exception is tomorrow's technical debt
- **Forces quality** - If a feature can't be demonstrated, should it exist?
- **Documentation by example** - Golden files ARE the documentation

### Why test-based enforcement instead of script?
- **Integrated with existing CI** - `cargo test` already runs
- **Rich assertion messages** - Tests can explain what's missing
- **Parallelizable** - Tests can check different categories concurrently
