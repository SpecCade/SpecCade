# Semantic Quality Lint System - Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add a semantic quality warning system that detects perceptual problems in generated assets and provides LLM-actionable fix suggestions.

**Problem:** LLMs generate specs that pass structural validation but produce bad-looking assets (clipping audio, muddy textures, wrong proportions, dissonant music). Current validation catches syntax errors, not aesthetic problems.

**Solution:** A new `speccade-lint` crate with domain-specific quality rules that run after generation, emitting tiered warnings (info/warning/error) with actionable fix guidance including spec paths and code templates.

**Tech Stack:** Rust (speccade-lint crate), integrates with speccade-cli

---

## Architecture

### New Crate Structure

```
crates/speccade-lint/
├── Cargo.toml
├── src/
│   ├── lib.rs           # Public API: lint(asset, rules) -> LintReport
│   ├── report.rs        # LintReport, LintIssue, Severity enum
│   ├── registry.rs      # RuleRegistry, rule enable/disable
│   ├── rules/
│   │   ├── mod.rs       # Rule trait definition
│   │   ├── audio.rs     # Audio quality rules
│   │   ├── texture.rs   # Texture quality rules
│   │   ├── mesh.rs      # Mesh quality rules
│   │   └── music.rs     # Music quality rules
│   └── suggestions.rs   # Fix template generation
```

### Integration Points

1. `speccade generate` calls lint after asset creation, appends to report.json
2. New `speccade lint` command for standalone quality checks
3. Editor calls lint API for real-time feedback in Problems panel

### Severity Levels

```rust
pub enum Severity {
    Info,     // Suggestions, stylistic preferences
    Warning,  // Likely problems, worth investigating
    Error,    // Definitely broken, should fail strict mode
}
```

### Exit Codes

| Code | Meaning |
|------|---------|
| 0 | No errors (warnings/info don't affect exit code) |
| 1 | One or more error-level issues found |
| 1 | With `--strict`: warnings also cause failure |

---

## Core Types

### Rule Trait

```rust
pub trait LintRule: Send + Sync {
    /// Unique identifier (e.g., "audio/clipping", "mesh/non-manifold")
    fn id(&self) -> &'static str;

    /// Human-readable description
    fn description(&self) -> &'static str;

    /// Which asset types this rule applies to
    fn applies_to(&self) -> &[AssetType];

    /// Default severity (can be overridden by config)
    fn default_severity(&self) -> Severity;

    /// Run the check, return issues found
    fn check(&self, asset: &AssetData, spec: Option<&Spec>) -> Vec<LintIssue>;
}
```

### LLM-Optimized LintIssue

```rust
pub struct LintIssue {
    pub rule_id: String,
    pub severity: Severity,
    pub message: String,

    // Location in generated asset (for humans/debugging)
    pub asset_location: Option<String>,  // "layer[0]", "bone:upper_arm_l"

    // Location in source spec (for LLM edits)
    pub spec_path: Option<String>,       // "recipe.params.layers[0].synthesis"

    // Measured values
    pub actual_value: Option<String>,
    pub expected_range: Option<String>,

    // Fix guidance (at least one should be present)
    pub suggestion: String,              // Human-readable explanation
    pub fix_template: Option<String>,    // Starlark snippet to insert/replace
    pub fix_delta: Option<f64>,          // Multiplier for numeric fixes
    pub fix_param: Option<String>,       // Specific param to adjust
}
```

### LintReport

```rust
pub struct LintReport {
    pub ok: bool,                    // true if no errors
    pub errors: Vec<LintIssue>,
    pub warnings: Vec<LintIssue>,
    pub info: Vec<LintIssue>,
    pub summary: LintSummary,
}

pub struct LintSummary {
    pub error_count: usize,
    pub warning_count: usize,
    pub info_count: usize,
}
```

---

## Rule Catalog

### Audio Rules (10 total)

**Error-level (3):**

| Rule ID | Description | Detection | Fix Guidance |
|---------|-------------|-----------|--------------|
| `audio/clipping` | Sample values exceed ±1.0 | Count samples > 1.0 or < -1.0 | `fix_delta`: peak/1.0, `fix_param`: amplitude |
| `audio/dc-offset` | Non-zero average sample value | Mean > 0.01 | `fix_template`: add highpass(cutoff=20) |
| `audio/silence` | Asset is entirely silent | RMS < -60dB | Check envelope attack or oscillator amplitude |

**Warning-level (5):**

| Rule ID | Description | Detection | Fix Guidance |
|---------|-------------|-----------|--------------|
| `audio/too-quiet` | Very low loudness | LUFS < -24 | `fix_delta`: target_lufs/actual_lufs |
| `audio/too-loud` | Near-clipping loudness | LUFS > -6 | `fix_template`: add limiter() |
| `audio/harsh-highs` | Excessive high-frequency energy | Energy > 50% above 8kHz | `fix_template`: lowpass(cutoff=6000) |
| `audio/muddy-lows` | Excessive low-mid buildup | Energy > 60% in 200-500Hz | `fix_template`: highpass(cutoff=80) |
| `audio/abrupt-end` | No fade-out, ends mid-waveform | Final 10ms amplitude > 0.1 | Increase envelope release |

**Info-level (2):**

| Rule ID | Description | Detection | Fix Guidance |
|---------|-------------|-----------|--------------|
| `audio/no-effects` | Dry signal, no spatial processing | Empty effects chain | `fix_template`: reverb() |
| `audio/mono-recommended` | Stereo file for short SFX | Stereo && duration < 2s | Use mono output format |

### Texture Rules (10 total)

**Error-level (3):**

| Rule ID | Description | Detection | Fix Guidance |
|---------|-------------|-----------|--------------|
| `texture/all-black` | Image is entirely black | Max pixel value < 5 | Check noise scale or color ramp |
| `texture/all-white` | Image is entirely white | Min pixel value > 250 | Check threshold or invert node |
| `texture/corrupt-alpha` | Alpha channel all 0 or 255 | Uniform alpha in RGBA | Check alpha source node |

**Warning-level (5):**

| Rule ID | Description | Detection | Fix Guidance |
|---------|-------------|-----------|--------------|
| `texture/low-contrast` | Narrow value range | Std dev < 20 | Increase color_ramp spread |
| `texture/banding` | Visible color stepping | < 32 unique values | Add dithering noise |
| `texture/tile-seam` | Edge discontinuity | Edge diff > threshold | Enable seamless mode |
| `texture/noisy` | Excessive high-frequency noise | High-freq > 70% | Reduce octaves or add blur |
| `texture/color-cast` | Strong single-channel dominance | Channel avg > 1.5× others | Balance color ramp |

**Info-level (2):**

| Rule ID | Description | Detection | Fix Guidance |
|---------|-------------|-----------|--------------|
| `texture/power-of-two` | Non-POT dimensions | Width/height not 2^n | Use 256/512/1024 |
| `texture/large-solid-regions` | >25% pixels identical | Histogram spike | Add subtle variation |

### Mesh Rules (12 total)

**Error-level (4):**

| Rule ID | Description | Detection | Fix Guidance |
|---------|-------------|-----------|--------------|
| `mesh/non-manifold` | Non-manifold edges | Edge shared by >2 faces | Remove duplicate faces |
| `mesh/degenerate-faces` | Zero-area triangles | Face area < 1e-8 | Remove or merge vertices |
| `mesh/unweighted-verts` | Vertices with no bone weights | Weight sum = 0 | Auto-weights or paint |
| `mesh/inverted-normals` | Normals pointing inward | Normal dot test | Recalculate normals |

**Warning-level (6):**

| Rule ID | Description | Detection | Fix Guidance |
|---------|-------------|-----------|--------------|
| `mesh/humanoid-proportions` | Limb ratios outside range | Arm/torso ratio check | `fix_delta` for bone length |
| `mesh/uv-overlap` | Overlapping UV islands | UV intersection test | Repack UVs |
| `mesh/uv-stretch` | High UV distortion | Area ratio > 2.0 | Adjust seams |
| `mesh/missing-material` | Faces with no material | Material index -1 | Assign material |
| `mesh/excessive-ngons` | >20% faces with >4 verts | Face vertex count | Triangulate |
| `mesh/isolated-verts` | Vertices not in any face | Vertex edge count = 0 | Remove isolated |

**Info-level (2):**

| Rule ID | Description | Detection | Fix Guidance |
|---------|-------------|-----------|--------------|
| `mesh/high-poly` | >50k triangles | Triangle count | Add LOD or decimate |
| `mesh/no-uvs` | Missing UV coordinates | UV layer check | Add UV projection |

### Music Rules (12 total)

**Error-level (3):**

| Rule ID | Description | Detection | Fix Guidance |
|---------|-------------|-----------|--------------|
| `music/empty-pattern` | Pattern has no notes | All cells empty | Add notes or remove |
| `music/invalid-note` | Note outside range | Note < C0 or > B9 | Transpose to valid range |
| `music/empty-arrangement` | No patterns in arrangement | Length = 0 | Add patterns |

**Warning-level (6):**

| Rule ID | Description | Detection | Fix Guidance |
|---------|-------------|-----------|--------------|
| `music/parallel-octaves` | Consecutive parallel octaves | Interval analysis | Use contrary motion |
| `music/parallel-fifths` | Consecutive parallel fifths | Interval analysis | Different intervals |
| `music/voice-crossing` | Lower voice above higher | Pitch comparison | Adjust voicing |
| `music/dense-pattern` | >8 simultaneous notes | Notes per row | Reduce density |
| `music/sparse-pattern` | <5% cell occupancy | Cell count | Add notes |
| `music/extreme-tempo` | BPM <40 or >300 | Tempo check | Adjust tempo |

**Info-level (3):**

| Rule ID | Description | Detection | Fix Guidance |
|---------|-------------|-----------|--------------|
| `music/unused-channel` | Channel with no notes | Channel analysis | Remove or add content |
| `music/no-variation` | Pattern repeats >4× | Arrangement analysis | Add B-section |
| `music/unresolved-tension` | Ends on dissonance | Final chord analysis | Resolve to tonic |

---

## CLI Interface

### Generate Integration

```bash
speccade generate --spec laser.star --out-root ./out

# Output includes lint results:
# Generated: sounds/laser.wav (0.5s)
#
# Quality issues:
#   WARNING audio/harsh-highs: High-frequency energy 62% (threshold: 50%)
#     at: layer[0]
#     suggestion: Add lowpass filter or reduce brightness
#
# Wrote: out/laser-01.report.json
```

### Standalone Lint Command

```bash
# Lint single asset
speccade lint --input sounds/laser.wav

# Lint directory (batch)
speccade lint --input-dir ./out --format json

# Strict mode (warnings fail)
speccade lint --input mesh.glb --strict

# Disable rules
speccade lint --input texture.png --disable-rule texture/power-of-two

# Only specific rules
speccade lint --input audio.wav --only-rules audio/clipping,audio/too-quiet

# With original spec (enables spec_path in output)
speccade lint --input audio.wav --spec audio.star
```

---

## Report.json Integration

```json
{
  "asset_id": "laser-01",
  "spec_hash": "abc123...",
  "generated_at": "2026-01-27T12:00:00Z",
  "outputs": [
    { "path": "sounds/laser.wav", "hash": "def456..." }
  ],
  "validation": {
    "ok": true,
    "errors": [],
    "warnings": []
  },
  "lint": {
    "ok": true,
    "errors": [],
    "warnings": [
      {
        "rule_id": "audio/harsh-highs",
        "severity": "warning",
        "message": "High-frequency energy 62% (threshold: 50%)",
        "asset_location": "frequency_spectrum[8000..22050]",
        "spec_path": "recipe.params.layers[0].synthesis",
        "actual_value": "62%",
        "expected_range": "< 50%",
        "suggestion": "Add lowpass filter or reduce brightness",
        "fix_template": "lowpass(cutoff=6000, resonance=0.5)",
        "fix_delta": null,
        "fix_param": null
      }
    ],
    "info": [],
    "summary": { "errors": 0, "warnings": 1, "info": 0 }
  }
}
```

---

## Implementation Tasks

### Phase 1: Infrastructure

**Task 1.1: Create speccade-lint crate scaffold**

Files:
- Create: `crates/speccade-lint/Cargo.toml`
- Create: `crates/speccade-lint/src/lib.rs`
- Modify: `Cargo.toml` (workspace members)

Deliverable: Empty crate that builds, added to workspace.

```bash
# Verify
cargo build -p speccade-lint
```

---

**Task 1.2: Implement core types**

Files:
- Create: `crates/speccade-lint/src/report.rs`
- Create: `crates/speccade-lint/src/rules/mod.rs`
- Modify: `crates/speccade-lint/src/lib.rs`

Implement:
- `Severity` enum
- `LintIssue` struct with all LLM-friendly fields
- `LintReport` struct
- `LintSummary` struct
- `LintRule` trait

```bash
# Verify
cargo test -p speccade-lint
```

---

**Task 1.3: Implement RuleRegistry**

Files:
- Create: `crates/speccade-lint/src/registry.rs`
- Modify: `crates/speccade-lint/src/lib.rs`

Implement:
- `RuleRegistry` struct
- `register()`, `lint()`, `disable_rule()`, `enable_only()` methods
- `default_rules()` constructor (empty for now)

```bash
# Verify
cargo test -p speccade-lint registry
```

---

**Task 1.4: Add lint CLI command**

Files:
- Create: `crates/speccade-cli/src/commands/lint.rs`
- Modify: `crates/speccade-cli/src/commands/mod.rs`
- Modify: `crates/speccade-cli/src/main.rs`
- Modify: `crates/speccade-cli/Cargo.toml` (add speccade-lint dep)

Implement:
- `speccade lint --input <file>` command
- `--strict`, `--disable-rule`, `--only-rules` flags
- `--format json` for machine output
- Exit code logic (0 = no errors, 1 = errors or strict+warnings)

```bash
# Verify
cargo run -p speccade-cli -- lint --help
```

---

**Task 1.5: Integrate lint into generate command**

Files:
- Modify: `crates/speccade-cli/src/commands/generate.rs`
- Modify: `crates/speccade-spec/src/report.rs` (add lint field)

Implement:
- Call lint after successful generation
- Print lint issues to console
- Add `lint` section to report.json

```bash
# Verify
cargo run -p speccade-cli -- generate --spec golden/starlark/minimal.star --out-root ./tmp
cat tmp/minimal.report.json | jq '.lint'
```

---

### Phase 2: Audio Rules

**Task 2.1: Implement audio rule infrastructure**

Files:
- Create: `crates/speccade-lint/src/rules/audio.rs`
- Modify: `crates/speccade-lint/src/rules/mod.rs`

Implement:
- `AudioRule` base helpers (load WAV, compute spectrum, etc.)
- Register audio rules in default_rules()

---

**Task 2.2: Implement error-level audio rules**

Files:
- Modify: `crates/speccade-lint/src/rules/audio.rs`

Implement:
- `audio/clipping` - detect samples > 1.0, compute fix_delta
- `audio/dc-offset` - compute mean, suggest highpass
- `audio/silence` - check RMS, suggest envelope check

Add tests with crafted WAV files that trigger each rule.

---

**Task 2.3: Implement warning-level audio rules**

Files:
- Modify: `crates/speccade-lint/src/rules/audio.rs`

Implement:
- `audio/too-quiet` - LUFS measurement
- `audio/too-loud` - LUFS measurement
- `audio/harsh-highs` - spectral analysis
- `audio/muddy-lows` - spectral analysis
- `audio/abrupt-end` - final sample check

---

**Task 2.4: Implement info-level audio rules**

Files:
- Modify: `crates/speccade-lint/src/rules/audio.rs`

Implement:
- `audio/no-effects` - requires spec context
- `audio/mono-recommended` - check channels + duration

---

### Phase 3: Texture Rules

**Task 3.1: Implement texture rule infrastructure**

Files:
- Create: `crates/speccade-lint/src/rules/texture.rs`
- Modify: `crates/speccade-lint/src/rules/mod.rs`

---

**Task 3.2: Implement error-level texture rules**

Implement: `texture/all-black`, `texture/all-white`, `texture/corrupt-alpha`

---

**Task 3.3: Implement warning-level texture rules**

Implement: `texture/low-contrast`, `texture/banding`, `texture/tile-seam`, `texture/noisy`, `texture/color-cast`

---

**Task 3.4: Implement info-level texture rules**

Implement: `texture/power-of-two`, `texture/large-solid-regions`

---

### Phase 4: Mesh Rules

**Task 4.1: Implement mesh rule infrastructure**

Files:
- Create: `crates/speccade-lint/src/rules/mesh.rs`
- Modify: `crates/speccade-lint/src/rules/mod.rs`

---

**Task 4.2: Implement error-level mesh rules**

Implement: `mesh/non-manifold`, `mesh/degenerate-faces`, `mesh/unweighted-verts`, `mesh/inverted-normals`

---

**Task 4.3: Implement warning-level mesh rules**

Implement: `mesh/humanoid-proportions`, `mesh/uv-overlap`, `mesh/uv-stretch`, `mesh/missing-material`, `mesh/excessive-ngons`, `mesh/isolated-verts`

---

**Task 4.4: Implement info-level mesh rules**

Implement: `mesh/high-poly`, `mesh/no-uvs`

---

### Phase 5: Music Rules

**Task 5.1: Implement music rule infrastructure**

Files:
- Create: `crates/speccade-lint/src/rules/music.rs`
- Modify: `crates/speccade-lint/src/rules/mod.rs`

---

**Task 5.2: Implement error-level music rules**

Implement: `music/empty-pattern`, `music/invalid-note`, `music/empty-arrangement`

---

**Task 5.3: Implement warning-level music rules**

Implement: `music/parallel-octaves`, `music/parallel-fifths`, `music/voice-crossing`, `music/dense-pattern`, `music/sparse-pattern`, `music/extreme-tempo`

---

**Task 5.4: Implement info-level music rules**

Implement: `music/unused-channel`, `music/no-variation`, `music/unresolved-tension`

---

### Phase 6: Documentation & Polish

**Task 6.1: Document all lint rules**

Files:
- Create: `docs/lint-rules.md`

Document each rule with:
- Rule ID
- Severity
- What it detects
- How to fix
- Example issue JSON

---

**Task 6.2: Add rules to stdlib dump**

Files:
- Modify: `crates/speccade-cli/src/commands/stdlib/mod.rs`

Add `lint_rules` section to stdlib dump output:
```json
{
  "lint_rules": [
    {
      "id": "audio/clipping",
      "severity": "error",
      "description": "...",
      "applies_to": ["audio"]
    }
  ]
}
```

---

**Task 6.3: Editor integration**

Files:
- Modify: `editor/src/components/ProblemsPanel.tsx` (or equivalent)
- Modify: `crates/speccade-editor/src/lib.rs`

Display lint issues in editor Problems panel alongside validation errors.

---

## Success Criteria

1. `speccade lint --input audio.wav` runs and outputs JSON with issues
2. `speccade generate` includes lint section in report.json
3. All 44 rules implemented (10 audio + 10 texture + 12 mesh + 12 music)
4. Each rule has at least one test
5. `--strict` flag causes warnings to fail
6. `--disable-rule` and `--only-rules` work correctly
7. LintIssue includes `spec_path` and `fix_template` where applicable
8. `docs/lint-rules.md` documents all rules
9. Stdlib dump includes lint rule catalog

---

## Future Enhancements (Out of Scope for v1)

- Custom user-defined rules via config file
- Rule severity overrides per-project
- Auto-fix mode (`speccade lint --fix`)
- Lint result caching by asset hash
- VLM-assisted verification for subjective quality (RFC-0010)
