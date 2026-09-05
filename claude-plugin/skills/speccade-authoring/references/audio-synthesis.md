# Audio synthesis authoring

Use `speccade stdlib dump --format json` and the current Rust types, not a copied JSON catalogue. Older examples used invented aliases (`fm`, `noise`, `frequency_sweep`, `end_frequency`) where current contracts use specific recipe variants and fields.

## Source map (repository-relative)

- `crates/speccade-spec/src/recipe/audio/mod.rs`: complete audio/layer structure.
- `crates/speccade-spec/src/recipe/audio/synthesis/`: synthesis variants, modulation and basic types.
- `crates/speccade-cli/src/commands/stdlib/audio.rs`: helper inventory/signatures.
- `docs/spec-reference/audio.md`: broader authoring guide; validate against Rust when it differs.
- `specs/audio/` and `packs/preset_library_v1/audio/`: reusable starting recipes.

Start from the runnable Starlark example in `../SKILL.md`. One layer first; add modulation or layers only after it validates and renders. Use explicit duration/sample rate/seed and gain headroom. Listen for clipping, discontinuities, unwanted tails and useful pitch/timbre, not just non-empty WAV data.

For ZX, source WAV must already be 22050 Hz mono signed 16-bit PCM. Tracker embedded-sample conversion is a separate path; see `nethercore-zx-integration.md`.
