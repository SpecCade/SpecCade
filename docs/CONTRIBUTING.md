# Contributing to SpecCade

Thank you for your interest in contributing to SpecCade! This guide covers development setup, testing, code style, and contribution workflow.

## Table of Contents

- [Development Setup](#development-setup)
- [Project Structure](#project-structure)
- [Running Tests](#running-tests)
- [Adding New Backends](#adding-new-backends)
- [Code Style](#code-style)
- [Contribution Workflow](#contribution-workflow)
- [Release Process](#release-process)

## Development Setup

### Prerequisites

- **Rust:** 1.70 or later ([rustup.rs](https://rustup.rs))
- **Blender:** 3.6 or later (for mesh/animation backends)
- **Git:** For version control
- **Python:** 3.10+ (optional, for migration tools)

### Clone the Repository

```bash
git clone https://github.com/SpecCade/SpecCade.git
cd SpecCade
```

### Build the Project

```bash
# Build all crates
cargo build

# Build with optimizations (for performance testing)
cargo build --release

# Install CLI locally
cargo install --path crates/speccade-cli
```

### Run the CLI

```bash
# Validate a spec
speccade validate --spec golden/speccade/specs/audio_sfx/laser_shot.json

# Generate an asset
speccade generate --spec golden/speccade/specs/audio_sfx/laser_shot.json --out-root ./test_output

# Check system requirements
speccade doctor
```

### IDE Setup

**VS Code (recommended):**

Install these extensions:

- `rust-analyzer` — Rust language server
- `Even Better TOML` — Cargo.toml syntax
- `Error Lens` — Inline errors
- `CodeLLDB` — Rust debugger

**VS Code settings (`.vscode/settings.json`):**

```json
{
  "rust-analyzer.checkOnSave.command": "clippy",
  "rust-analyzer.cargo.features": "all",
  "editor.formatOnSave": true
}
```

## Project Structure

```
SpecCade/
├── crates/
│   ├── speccade-spec/          # Core spec types, validation, hashing
│   ├── speccade-cli/           # CLI binary (speccade command)
│   ├── speccade-backend-audio/ # Audio SFX backend (Rust)
│   ├── speccade-backend-instrument/ # Instrument backend (Rust)
│   ├── speccade-backend-music/ # Music/tracker backend (Rust)
│   ├── speccade-backend-texture/ # Texture backend (Rust)
│   ├── speccade-backend-normal/ # Normal map backend (Rust)
│   └── speccade-backend-blender/ # Blender orchestrator (Rust + Python)
├── blender/
│   └── entrypoint.py           # Blender script for mesh/animation generation
├── golden/
│   ├── legacy/                 # Legacy reference specs (optional)
│   └── speccade/               # Canonical specs and golden outputs
│       ├── specs/              # JSON specs (source of truth)
│       └── expected/           # Expected hashes and metrics
├── docs/
│   ├── rfcs/                   # RFCs and design docs
│   ├── DETERMINISM.md          # Determinism policy
│   ├── MIGRATION.md            # Migration guide
│   ├── SPEC_REFERENCE.md       # Spec schema reference
│   └── CONTRIBUTING.md         # This file
├── schemas/
│   └── speccade-spec-v1.schema.json # JSON Schema for validation
├── Cargo.toml                  # Workspace manifest
└── README.md                   # Project readme
```

### Crate Responsibilities

| Crate | Responsibility |
|-------|---------------|
| `speccade-spec` | Spec types, validation, hashing, report generation |
| `speccade-cli` | CLI argument parsing, command dispatch |
| `speccade-backend-*` | Asset generation backends (Rust or Blender-based) |

## Running Tests

### Unit Tests

```bash
# Run all tests
cargo test

# Run tests for a specific crate
cargo test -p speccade-spec

# Run tests with output
cargo test -- --nocapture

# Run tests in release mode (faster for DSP tests)
cargo test --release
```

### Golden Corpus Tests

Golden corpus tests validate deterministic output against recorded hashes/metrics.

```bash
# Run golden tests (Tier 1: audio, music, texture)
cargo test --test golden_tier1

# Run golden tests (Tier 2: meshes, animations)
cargo test --test golden_tier2

# Update golden outputs (after intentional changes)
cargo test --test golden_tier1 -- --ignored
```

**Golden test workflow:**

1. Make changes to backend implementation
2. Run golden tests: `cargo test --test golden_tier1`
3. If tests fail due to intentional changes:
   - Review diffs to confirm correctness
   - Update golden outputs: `cargo test --test golden_tier1 -- --ignored`
   - Commit updated hashes/metrics
4. If tests fail due to bugs, fix the backend and re-test

### Integration Tests

```bash
# Run CLI integration tests
cargo test --test cli_integration

# Test migration tool
cargo test --test migration
```

### Performance Benchmarks

```bash
# Run benchmarks (requires nightly Rust)
cargo +nightly bench
```

## Adding New Backends

### Overview

Backends implement asset generation for specific recipe kinds. Rust backends are preferred for determinism (Tier 1). Blender backends are acceptable for mesh/animation tasks where metric validation suffices (Tier 2).

### Steps to Add a New Backend

#### 1. Define the Recipe Kind

Update `crates/speccade-spec/src/recipe/mod.rs`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", content = "params")]
pub enum Recipe {
    // Existing kinds...

    #[serde(rename = "audio_sfx.granular_synth_v1")]
    AudioSfxGranularSynth(AudioSfxGranularSynthParams),
}
```

#### 2. Define Recipe Params

Create `crates/speccade-spec/src/recipe/audio_sfx.rs` (or extend existing):

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioSfxGranularSynthParams {
    pub duration_seconds: f32,
    pub sample_rate: u32,
    pub grain_size_ms: f32,
    pub grain_density: f32,
    pub pitch_variance: f32,
    // ... more params
}
```

#### 3. Create the Backend Crate

```bash
cargo new --lib crates/speccade-backend-granular
```

Add to workspace `Cargo.toml`:

```toml
[workspace]
members = [
    "crates/speccade-spec",
    "crates/speccade-cli",
    "crates/speccade-backend-granular",
    # ...
]
```

#### 4. Implement the Backend

`crates/speccade-backend-granular/src/lib.rs`:

```rust
use speccade_spec::{AudioSfxGranularSynthParams, ReportBuilder};

pub fn generate(
    params: &AudioSfxGranularSynthParams,
    seed: u32,
    output_path: &Path,
) -> Result<ReportBuilder, BackendError> {
    // 1. Initialize RNG with seed (use PCG32)
    let mut rng = Pcg32::seed_from_u64(seed as u64);

    // 2. Generate audio samples
    let samples = generate_samples(params, &mut rng);

    // 3. Write WAV file (deterministic, no timestamps)
    write_wav(output_path, &samples, params.sample_rate)?;

    // 4. Compute artifact hash
    let hash = hash_wav_pcm(output_path)?;

    // 5. Build report
    let mut report = ReportBuilder::new();
    report.add_output(OutputResult {
        kind: OutputKind::Audio,
        format: OutputFormat::Wav,
        path: output_path.to_path_buf(),
        hash: Some(hash),
        metrics: None,
    });

    Ok(report)
}
```

#### 5. Register Backend in CLI

Update `crates/speccade-cli/src/dispatch.rs`:

```rust
pub fn dispatch_generate(spec: &SpecCadeSpec, out_root: &Path) -> Result<Report> {
    match &spec.recipe.kind {
        RecipeKind::AudioSfxLayeredSynth(params) => {
            speccade_backend_audio::generate(params, spec.seed, out_root)
        }
        RecipeKind::AudioSfxGranularSynth(params) => {
            speccade_backend_granular::generate(params, spec.seed, out_root)
        }
        // ...
    }
}
```

#### 6. Add Golden Tests

Create test specs in `golden/speccade/specs/audio_sfx/granular_*.json`.

Generate golden outputs:

```bash
speccade generate --spec golden/speccade/specs/audio_sfx/granular_demo.json --out-root golden/speccade/outputs
```

Record hashes:

```bash
# Hash the WAV PCM data (skip RIFF header)
blake3sum golden/speccade/outputs/granular_demo.wav > golden/speccade/expected/hashes/audio_sfx/granular_demo.hash
```

Add test case in `tests/golden_tier1.rs`:

```rust
#[test]
fn test_granular_demo() {
    test_golden_spec(
        "golden/speccade/specs/audio_sfx/granular_demo.json",
        "golden/speccade/expected/hashes/audio_sfx/granular_demo.hash",
    );
}
```

#### 7. Update Documentation

- Add recipe kind to `docs/SPEC_REFERENCE.md`
- Add example spec to RFC-0001 if it demonstrates a new pattern
- Update README.md asset types table

### Blender Backend Guidelines

For Blender-backed recipes:

1. Add recipe handling to `blender/entrypoint.py`
2. Use only stdlib (no external Python deps)
3. Return metric JSON (triangle count, bounds, etc.)
4. Add metric validation in `tests/golden_tier2.rs`

## Code Style

### Rust Style

Follow standard Rust conventions:

```bash
# Format code
cargo fmt

# Lint code
cargo clippy -- -D warnings

# Check without building
cargo check
```

**Key conventions:**

- Use `rustfmt` defaults (120-char lines, 4-space indent)
- Prefer explicit types over `impl Trait` in public APIs
- Use `?` for error propagation
- Document public APIs with `///` doc comments
- Avoid `unwrap()` in production code (use `expect()` with clear messages)

### Python Style (Blender Scripts)

```bash
# Format Python code
black blender/

# Lint Python code
ruff check blender/
```

**Key conventions:**

- Follow PEP 8
- Use type hints where practical
- Stdlib only (no external dependencies)
- Document functions with docstrings

### Commit Style

Use conventional commits:

```
feat(backend-audio): add granular synthesis support
fix(cli): correct output path validation
docs(spec): clarify seed derivation policy
test(golden): add mesh golden tests
refactor(spec): simplify recipe enum
```

**Types:** `feat`, `fix`, `docs`, `test`, `refactor`, `perf`, `chore`

## Contribution Workflow

### 1. Open an Issue

Before starting work, open an issue to discuss:

- Feature proposals
- Bug reports
- API changes

For small fixes (typos, docs), you can skip this step.

### 2. Fork and Branch

```bash
# Fork the repo on GitHub, then:
git clone https://github.com/YOUR_USERNAME/SpecCade.git
cd SpecCade
git remote add upstream https://github.com/SpecCade/SpecCade.git

# Create a feature branch
git checkout -b feat/granular-synthesis
```

### 3. Make Changes

- Write tests first (TDD recommended)
- Implement the feature
- Run tests: `cargo test`
- Format code: `cargo fmt`
- Lint code: `cargo clippy`
- Update documentation

### 4. Commit

```bash
git add .
git commit -m "feat(backend-audio): add granular synthesis"
```

### 5. Push and Open PR

```bash
git push origin feat/granular-synthesis
```

Open a pull request on GitHub. Include:

- Description of changes
- Link to related issue
- Test results
- Breaking changes (if any)

### 6. Code Review

Maintainers will review your PR. Address feedback by pushing new commits:

```bash
git add .
git commit -m "fix: address review feedback"
git push origin feat/granular-synthesis
```

### 7. Merge

Once approved, a maintainer will merge your PR. Thank you!

## Release Process

SpecCade follows semantic versioning: `MAJOR.MINOR.PATCH`

- **MAJOR:** Breaking changes to spec schema or CLI
- **MINOR:** New features, backward-compatible
- **PATCH:** Bug fixes, docs, performance

### Release Checklist (Maintainers)

1. Update version in all `Cargo.toml` files
2. Update `CHANGELOG.md`
3. Run full test suite: `cargo test --release`
4. Run golden tests: `cargo test --test golden_tier1 --test golden_tier2`
5. Build release binaries: `cargo build --release`
6. Tag release: `git tag v1.0.0 && git push origin v1.0.0`
7. Publish to crates.io: `cargo publish -p speccade-spec && cargo publish -p speccade-cli`
8. Create GitHub release with binaries and changelog

## Getting Help

- **Questions:** [GitHub Discussions](https://github.com/SpecCade/SpecCade/discussions)
- **Bugs:** [GitHub Issues](https://github.com/SpecCade/SpecCade/issues)
- **Chat:** [Discord](https://discord.gg/speccade) (coming soon)

## Code of Conduct

Be respectful, inclusive, and collaborative. We follow the [Contributor Covenant](https://www.contributor-covenant.org/).

## License

By contributing, you agree that your contributions will be licensed under the MIT License.

---

Thank you for contributing to SpecCade! Your work helps game developers build better tools and workflows.
