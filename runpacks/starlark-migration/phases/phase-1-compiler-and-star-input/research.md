# Phase 1 Research: Findings

## Overview

This document answers the research questions from the Phase 1 prompt, based on analysis of the existing SpecCade codebase.

---

## 1. CLI Structure

### How does the CLI dispatch commands?

**Entry point:** `crates/speccade-cli/src/main.rs`

The CLI uses `clap` for command parsing with a `Commands` enum defining subcommands:

```rust
#[derive(Subcommand)]
enum Commands {
    Validate { spec: String, artifacts: bool },
    Generate { spec: String, out_root: Option<String>, expand_variants: bool },
    GenerateAll { ... },
    Preview { ... },
    Doctor,
    Expand { spec: String },
    Migrate { ... },
    Fmt { ... },
    Template { ... },
}
```

Dispatch happens in `main()` via pattern matching:

```rust
let result = match cli.command {
    Commands::Validate { spec, artifacts } => commands::validate::run(&spec, artifacts),
    Commands::Generate { spec, out_root, expand_variants } =>
        commands::generate::run(&spec, out_root.as_deref(), expand_variants),
    // ... etc
};
```

### Where does JSON spec loading happen?

JSON spec loading is done **inline in each command module**. There is no abstracted input layer.

**Example from `commands/validate.rs` (lines 30-36):**

```rust
// Read spec file
let spec_content = fs::read_to_string(spec_path)
    .with_context(|| format!("Failed to read spec file: {}", spec_path))?;

// Parse spec
let spec = Spec::from_json(&spec_content)
    .with_context(|| format!("Failed to parse spec file: {}", spec_path))?;
```

The same pattern appears in:
- `commands/generate.rs` (lines 39-44)
- `commands/expand.rs` (lines 12-18)
- `commands/fmt.rs`
- `commands/generate_all.rs`

**Key insight:** Each command reads a file path string, reads the file as a string, and calls `Spec::from_json()`. This is an ideal abstraction point for input dispatch.

### How are validate/generate commands structured?

Both commands follow a similar flow:

1. Read spec file from disk (`fs::read_to_string`)
2. Parse JSON into `Spec` (`Spec::from_json`)
3. Compute hashes (`canonical_spec_hash`, `canonical_recipe_hash`)
4. Run validation (`validate_spec` or `validate_for_generate`)
5. For generate: dispatch to backend (`dispatch_generate`)
6. Build and write report (`ReportBuilder`)

**Backend dispatch:** `crates/speccade-cli/src/dispatch/mod.rs`

```rust
pub fn dispatch_generate(
    spec: &Spec,
    out_root: &str,
    spec_path: &Path,
) -> Result<Vec<OutputResult>, DispatchError> {
    let recipe = spec.recipe.as_ref().ok_or(DispatchError::NoRecipe)?;
    let kind = &recipe.kind;

    match kind.as_str() {
        "audio_v1" => audio::generate_audio(spec, out_root_path),
        "music.tracker_song_v1" => music::generate_music(spec, out_root_path, spec_dir),
        // ... etc
    }
}
```

---

## 2. Spec Crate

### What is the `Spec` struct shape?

**Location:** `crates/speccade-spec/src/spec.rs` (lines 92-145)

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Spec {
    pub spec_version: u32,           // Must be 1
    pub asset_id: String,            // [a-z][a-z0-9_-]{2,63}
    pub asset_type: AssetType,       // audio, music, texture, etc.
    pub license: String,             // SPDX identifier
    pub seed: u32,                   // RNG seed
    pub outputs: Vec<OutputSpec>,    // Required output artifacts

    // Optional fields
    pub description: Option<String>,
    pub style_tags: Option<Vec<String>>,
    pub engine_targets: Option<Vec<EngineTarget>>,
    pub migration_notes: Option<Vec<String>>,
    pub variants: Option<Vec<VariantSpec>>,
    pub recipe: Option<Recipe>,      // Required for generate
}
```

**Recipe struct:** `crates/speccade-spec/src/recipe/mod.rs` (lines 103-119)

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Recipe {
    pub kind: String,               // e.g., "audio_v1", "texture.procedural_v1"
    pub params: serde_json::Value,  // Backend-specific parameters
}
```

### How is validation invoked?

**Location:** `crates/speccade-spec/src/validation/mod.rs`

Two main entry points:

1. **`validate_spec(spec: &Spec) -> ValidationResult`** (line 71)
   - Contract-only validation (no recipe required)
   - Validates: spec_version, asset_id, seed, outputs, recipe compatibility

2. **`validate_for_generate(spec: &Spec) -> ValidationResult`** (line 248)
   - Full validation including recipe requirement
   - Adds: E010 (missing recipe), E011 (unsupported recipe kind)

Validation is modular:
- `validation/common.rs` - Numeric range helpers
- `validation/path_safety.rs` - Output path validation
- `validation/recipe_outputs.rs` - Output/recipe compatibility
- `validation/recipe_outputs_*.rs` - Backend-specific validation

### How is hashing (ir_hash) computed?

**Location:** `crates/speccade-spec/src/hash.rs`

```rust
pub fn canonical_spec_hash(spec: &Spec) -> Result<String, SpecError> {
    let value = spec.to_value()?;  // Convert to serde_json::Value
    canonical_value_hash(&value)
}

pub fn canonical_value_hash(value: &serde_json::Value) -> Result<String, SpecError> {
    let canonical = canonicalize_json(value)?;  // RFC 8785 JCS
    let hash = blake3::hash(canonical.as_bytes());
    Ok(hash.to_hex().to_string())
}
```

**Canonicalization rules (RFC 8785 JCS):**
- Object keys sorted lexicographically
- No whitespace between tokens
- Numbers formatted per IEEE 754
- Strings use minimal escaping

**Output:** 64-character lowercase hexadecimal BLAKE3 hash.

---

## 3. Integration Points

### Where should Starlark parsing hook in?

**Recommended integration point:** Before `Spec::from_json()` in each command.

The natural abstraction is an **input layer** that:
1. Detects file extension (`.json` vs `.star`)
2. For `.json`: Load and parse directly
3. For `.star`: Evaluate Starlark -> get JSON-like value -> convert to `Spec`

**Proposed function signature:**

```rust
pub fn load_spec(path: &Path) -> Result<Spec, InputError> {
    match path.extension().and_then(|e| e.to_str()) {
        Some("json") => load_json_spec(path),
        Some("star") | Some("bzl") => load_starlark_spec(path),
        _ => Err(InputError::UnknownExtension(path.to_owned())),
    }
}
```

This could live in a new `speccade-compiler` crate or within `speccade-cli`.

### Can input dispatch be abstracted?

Yes. Currently, input loading is duplicated across commands:
- `validate.rs`, `generate.rs`, `expand.rs`, `fmt.rs`, `generate_all.rs`

A centralized `load_spec()` function would:
1. Reduce code duplication
2. Provide a single integration point for Starlark
3. Handle source provenance tracking (`source_kind`, `source_hash`)

### Are there existing expansion passes?

**Yes.** The `music.tracker_song_compose_v1` recipe has an expansion pass.

**Location:** `crates/speccade-backend-music/src/compose/mod.rs`

```rust
pub fn expand_compose(
    params: &MusicTrackerSongComposeV1Params,
    seed: u32,
) -> Result<MusicTrackerSongV1Params> {
    // Expands Pattern IR to canonical tracker_song_v1 params
}
```

**CLI integration:** `commands/expand.rs`

```rust
match recipe.kind.as_str() {
    "music.tracker_song_compose_v1" => {
        let params = recipe.as_music_tracker_song_compose()?;
        let expanded = speccade_backend_music::expand_compose(&params, spec.seed)?;
        let json = serde_json::to_string_pretty(&expanded)?;
        println!("{}", json);
    }
}
```

**Dispatch integration:** `dispatch/music.rs` (line 31-32)

```rust
let expanded = speccade_backend_music::expand_compose(&params, spec.seed)?;
generate_music_from_params(&expanded, ...)
```

This demonstrates the pattern for authoring-level expansions:
1. Parse high-level params
2. Expand to canonical params
3. Pass canonical params to backend

Starlark would fit the same model: Starlark -> canonical `Spec` JSON.

---

## 4. Dependencies

### What Starlark crate should be used?

**Recommended:** [`starlark`](https://crates.io/crates/starlark) (facebook/starlark-rust)

**Key facts:**
- Maintained by Meta (Facebook), used in Buck2
- Latest version: 0.12.x (as of research date)
- Licensed: Apache 2.0

**Related crates:**
- `starlark_derive` - Proc macros
- `starlark_map` - Memory-efficient maps/sets
- `starlark_syntax` - AST and parsing

### What safety limits does it support?

**Language-level safety (built-in):**
- No file/network access (sandboxed by design)
- No recursion by default (can be enabled as extension)
- Deterministic execution
- Garbage collected heap

**Runtime limits (NOT built-in to the crate):**

The `starlark` crate does **not** provide configurable memory or CPU time limits. Safety must be enforced externally:

1. **Memory limits:** Use system-level limits (ulimit) or wrap with `cap` crate
2. **Time limits:** Implement via external timeout (e.g., `tokio::time::timeout`)
3. **Instruction limits:** Not directly supported; would require custom evaluator hooks

**Starlark extensions:**
- Type annotations (optional runtime checking)
- Recursion (disabled by default)
- Top-level `for` loops

**Integration pattern:**

```rust
use starlark::environment::{Globals, Module};
use starlark::eval::Evaluator;
use starlark::syntax::{AstModule, Dialect};

fn eval_starlark(source: &str) -> Result<serde_json::Value> {
    let ast = AstModule::parse("spec.star", source.to_string(), &Dialect::Standard)?;
    let globals = Globals::standard();
    let module = Module::new();
    let mut eval = Evaluator::new(&module);

    let result = eval.eval_module(ast, &globals)?;
    // Convert result to JSON value
}
```

---

## 5. File Paths Summary

| Purpose | Path |
|---------|------|
| CLI entry | `crates/speccade-cli/src/main.rs` |
| CLI commands | `crates/speccade-cli/src/commands/*.rs` |
| Backend dispatch | `crates/speccade-cli/src/dispatch/mod.rs` |
| Spec struct | `crates/speccade-spec/src/spec.rs` |
| Recipe struct | `crates/speccade-spec/src/recipe/mod.rs` |
| Validation | `crates/speccade-spec/src/validation/mod.rs` |
| Hashing | `crates/speccade-spec/src/hash.rs` |
| Compose expansion | `crates/speccade-backend-music/src/compose/mod.rs` |
| Workspace root | `Cargo.toml` |
| CLI Cargo.toml | `crates/speccade-cli/Cargo.toml` |

---

## 6. Recommended Approach

Based on this research, the recommended Phase 1 implementation:

1. **Create input abstraction** in `speccade-cli`:
   - New `input.rs` module with `load_spec(path: &Path) -> Result<Spec>`
   - Detect `.json` vs `.star` by extension
   - Refactor commands to use this abstraction

2. **Add Starlark evaluation** (new crate or module):
   - Depend on `starlark` crate
   - Evaluate `.star` files to JSON-like output
   - Convert to `Spec` using existing `Spec::from_value()`

3. **Preserve determinism**:
   - Hash computation remains on canonical IR (post-Starlark eval)
   - Add `source_kind` and `source_hash` to reports for provenance

4. **Implement safety limits externally**:
   - Use `tokio::time::timeout` for CPU limits
   - Consider `cap` crate for memory limits if needed

5. **Incremental rollout**:
   - Phase 1a: Input abstraction + JSON passthrough
   - Phase 1b: Starlark evaluation with minimal stdlib
   - Phase 1c: Safety limits and diagnostics
