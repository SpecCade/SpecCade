# SpecCade TODOs (Curated Backlog)

This is a curated backlog of *future work*. For what’s implemented today, see `PARITY_MATRIX.md` and `docs/spec-reference/`.

## Music (Compose / Pattern IR)

- Add/expand snapshot tests for `docs/examples/music/*.expanded.params.json` via `speccade expand`.
- Extend Pattern IR operators and hard limits where needed (RFC-0003 / RFC-0004), keeping determinism and reviewability.
- Improve `speccade expand` UX for review workflows (stable formatting, optional file output, diff-friendly mode).
- Tighten XM/IT parity checks and document known differences (looping, envelopes, effect coverage).

## Music (Content / Workflow)

- Prefer `TrackerInstrument.ref` (external `audio_v1` specs) in music examples/goldens so instruments are reusable and specs stay small; keep inline synthesis only when demonstrating a feature.
- Add a higher-quality drum example (filtered noise / metallic synthesis and/or WAV drum samples) and tune it by ear so kick/snare/hat sound correct in common XM/IT players.
- Add a small “XM vs IT parity” listening checklist and automate basic structural checks (loop flags/envelope flags) in tests.
- Grow “genre kits” as data packages: curated compose `defs` + instrument refs + timebase/harmony defaults (with workflows that focus edits on patterns, not instruments).

## Audio

- Keep expanding and tuning the `audio_v1` preset library (kick/snare/hat/clap/bass/lead/pad/etc.) and gate changes with QA metrics (no clipping, sane RMS, low DC offset).
- Add an “audio audit” report (peak/RMS/DC offset) to catch broken presets and regressions.
- Improve loop-point generation + click-free envelope defaults for tracker instrument baking; document best practices.

## LLM / Prompt-to-Spec Workflows

- See `docs/LLM_PROMPT_TO_ASSET_ROADMAP.md` for a concrete execution path, example JSON schemas, and remaining friction points.
- Add a machine-readable stdlib/API dump (e.g. `speccade stdlib dump --format json`) so tools/LLMs can ground against the real function set, enums, and parameter ranges.
- Add `--json` output for `eval/validate/generate` errors and warnings (stable codes + file/line/col + suggested fixes) to enable reliable auto-repair loops.
- Generalize `speccade template` beyond textures (audio/music/mesh) and add “preset + overrides” patterns to reduce spec boilerplate for generators.
- Add higher-level Starlark constructors (e.g. `audio_spec()`, `texture_spec()`, `mesh_spec()`) similar to `music_spec()` so LLMs emit fewer raw recipe dicts.
- Implement `preview` for fast iteration (waveform/spectrogram thumbnails, texture previews, GLB viewer hooks) and/or emit preview artifacts as part of generation.
- Add an `analyze` command that outputs quality metrics suitable for iteration (audio peak/RMS/DC/clipping; texture tiling/artifacts/contrast; music structural checks; mesh bounds/topology budgets).
- Add a content-addressed cache keyed by canonical spec/recipe hash + toolchain/backend versions to make iterative generate loops cheap.
- Add a Nethercore-oriented budget profile (e.g. enforce 22050 Hz sample rate without implying “8-bit” constraints) and document recommended profiles per target runtime.

## Tooling / QA

- Grow the Tier-1 golden corpus (`golden/`) for audio/music/textures and run it in CI.
- Add “expand/inspect” style commands/flags where helpful for review (compose → expanded JSON, packed-texture intermediate maps).
