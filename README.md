# SpecCade

> **Specs in. Assets out.**

SpecCade is a declarative asset generation system that transforms JSON specifications into game-ready assets. Write a spec once, generate consistently every time.

## What is SpecCade?

SpecCade turns structured JSON specs into production assets:

- **Audio:** WAV sound effects and instrument samples
- **Music:** XM/IT tracker modules with inline synthesis
- **Textures:** PNG material maps (albedo, roughness, metallic, normal, AO, emissive)
- **Meshes:** GLB static meshes from parameterized primitives
- **Skeletal Meshes:** GLB skinned meshes with skeletons and auto-weights
- **Animations:** GLB skeletal animation clips

### Why SpecCade?

**Declarative** — Describe what you want, not how to make it. Specs are pure data, not code.

**Deterministic** — Same spec + seed = identical output. Perfect for version control and build pipelines.

**Portable** — JSON specs work everywhere. No runtime dependencies, no platform lock-in.

**Safe** — Pure data specs with no code execution. Review, diff, and merge with confidence.

## Quick Start

### Installation

```bash
# From source (requires a recent Rust toolchain)
git clone https://github.com/SpecCade/SpecCade.git
cd SpecCade
cargo install --path crates/speccade-cli

# Verify installation
speccade --version
speccade doctor
```

### Validate a Spec

```bash
speccade validate --spec my_sound.json
```

`validate` writes an `${asset_id}.report.json` file next to the spec file.

### Generate Assets

```bash
speccade generate --spec my_sound.json --out-root ./output
```

Assets are written under `./output/`. A `${asset_id}.report.json` file (hashes/metrics/validation) is written next to the spec file.

## Example Spec

Here's a simple laser sound effect:

```json
{
  "spec_version": 1,
  "asset_id": "laser_shot",
  "asset_type": "audio",
  "license": "CC0-1.0",
  "seed": 1002,
  "description": "FM synthesis laser shot - classic arcade sound",
  "outputs": [
    {
      "kind": "primary",
      "format": "wav",
      "path": "laser_shot.wav"
    }
  ],
  "recipe": {
    "kind": "audio_v1",
    "params": {
      "duration_seconds": 0.25,
      "sample_rate": 44100,
      "layers": [
        {
          "synthesis": {
            "type": "fm_synth",
            "carrier_freq": 1200.0,
            "modulator_freq": 3000.0,
            "modulation_index": 8.0,
            "freq_sweep": { "end_freq": 300.0, "curve": "exponential" }
          },
          "envelope": {
            "attack": 0.001,
            "decay": 0.1,
            "sustain": 0.3,
            "release": 0.1
          },
          "volume": 0.9,
          "pan": 0.0
        }
      ]
    }
  }
}
```

Run `speccade generate --spec laser_shot.json` and you get a deterministic `laser_shot.wav` every time.

## CLI Commands

### Core Commands

```bash
# Validate a spec without generating assets
speccade validate --spec <path>

# Generate assets from a spec (requires `recipe`)
speccade generate --spec <path> --out-root <path>

# Generate all specs in a directory
speccade generate-all --spec-dir <dir> --out-root <dir>

# Preview 3D assets (Blender-backed assets only). Use `--gif` to export an animated GIF preview.
# Default GIF filename: <asset_id>.preview.gif (written next to the spec file)
speccade preview --spec <path> --out-root <path>
speccade preview --spec path/to/spec.json --gif --out-root ./output
speccade preview --spec path/to/spec.json --gif --out my-preview.gif --fps 24 --scale 2 --out-root ./output

# Generate a 6-view validation grid PNG for visual LLM verification.
# Views: FRONT, BACK, TOP, LEFT, RIGHT, ISO (isometric)
# Useful for Claude vision to validate 3D asset correctness against spec comments.
speccade preview-grid --spec <path>
speccade preview-grid --spec path/to/mesh.star --out grid.png --panel-size 512

# Format a spec to canonical JSON style
speccade fmt --spec <path>

# Check system requirements and dependencies
speccade doctor

# Expand compose music specs into canonical tracker params JSON
speccade expand --spec <path>

# List and copy texture templates
speccade template list --asset-type texture
speccade template copy preset_texture_material_set_basic --to ./specs/texture/my_material.json
```

## Asset Types

Audio, music, textures, sprites, UI, fonts, static meshes, skeletal meshes, and animations. See [`docs/spec-reference/README.md`](docs/spec-reference/README.md) for the full list with recipe kinds, and [`PARITY_MATRIX.md`](PARITY_MATRIX.md) for backend coverage.

## Determinism

Same spec + seed = identical output. See [`docs/DETERMINISM.md`](docs/DETERMINISM.md) for the full determinism policy and [`PARITY_MATRIX.md`](PARITY_MATRIX.md) for per-backend tier guarantees.

## Documentation

- **[Docs Map](docs/README.md)** — What to read first
- **[Spec Reference](docs/spec-reference/README.md)** — Canonical contract + per-asset reference
- **[Determinism Policy](docs/DETERMINISM.md)** — RNG, hashing, and validation rules
- **[Contributing](docs/CONTRIBUTING.md)** — Development setup and contribution guidelines

## Use Cases

### Build Pipelines

Commit specs to version control. Generate assets as part of your CI/CD pipeline. Track changes through diffs.

### Procedural Variation

The `variants` field can be expanded by the CLI during generation with `speccade generate --expand-variants`.
Variants are generated under `{out_root}/variants/{variant_id}/` using derived seeds.

```json
{
  "variants": [
    { "variant_id": "soft", "seed_offset": 0 },
    { "variant_id": "hard", "seed_offset": 100 }
  ]
}
```

### Rapid Prototyping

Generate placeholder assets fast. Iterate by tweaking params and regenerating instantly.

### Asset Libraries

Build reusable spec libraries. Share specs as JSON. Generate locally or in the cloud.

## Requirements

- **Rust:** stable toolchain (CI uses `dtolnay/rust-toolchain@stable`)
- **Blender:** 3.6+ (for mesh/character/animation generation)

Run `speccade doctor` to check your environment.

## Development Status

SpecCade is in active development targeting v1.0:

- **v0.1:** Core validation and generation for audio/music/texture
- **v0.2:** Full asset type coverage (meshes, characters, animations)
- **v0.3:** Migration tooling and golden corpus CI
- **v1.0:** Stable spec contract, production-ready

## License

SpecCade is licensed under MIT. Generated assets inherit the license specified in their spec.

## Contributing

See [`docs/CONTRIBUTING.md`](docs/CONTRIBUTING.md) for development setup, testing, and contribution guidelines.

## Links

- **Repository:** [github.com/SpecCade/SpecCade](https://github.com/SpecCade/SpecCade)
- **Issues:** [github.com/SpecCade/SpecCade/issues](https://github.com/SpecCade/SpecCade/issues)
- **Discussions:** [github.com/SpecCade/SpecCade/discussions](https://github.com/SpecCade/SpecCade/discussions)

---

Built with care for game developers who value reproducibility, portability, and control.
